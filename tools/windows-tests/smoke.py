"""Run an isolated Windows Server smoke test against a local OpenAI-compatible mock."""

import argparse
import ctypes
from ctypes import wintypes
import http.server
import json
import msvcrt
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import threading
import time
import uuid
import zlib

sys.stdout.reconfigure(encoding="utf-8")


REPO = Path(__file__).resolve().parents[2]
TOOLS = REPO / "target/validation"
PIPE_NAME = r"\\.\pipe\ctrllingo"
SESSION = 63001
WATCHDOG_BASE_SECONDS = 180
LANGUAGES = {
    "en": "This is a local English translation.",
    "ja": "これはローカルの日本語訳です。",
    "es": "Esta es una traducción local al español.",
    "vi": "Đây là bản dịch tiếng Việt chạy cục bộ.",
    "de": "Dies ist eine lokale deutsche Übersetzung.",
}


class MockTranslation(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        owner = self.server
        if self.headers.get("Authorization") != "Bearer local-test-key":
            self.send_error(401)
            return
        if self.path.rstrip("/") not in ("/v1/chat/completions", "/chat/completions"):
            self.send_error(404)
            return
        try:
            request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            user = json.loads(request["messages"][1]["content"])
            language = user.get("target_language")
            if language not in LANGUAGES:
                raise ValueError(f"unexpected target language: {language!r}")
            if not isinstance(user.get("text"), str) or not user["text"].strip():
                raise ValueError("translation request has no source text")
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            self.send_error(400, str(error))
            return

        with owner.lock:
            owner.calls.append({"target_language": language, "text": user["text"]})
            call_number = len(owner.calls)
            owner.changed.notify_all()
        owner.release.wait(20)
        body = json.dumps({
            "id": f"local-test-{call_number}",
            "object": "chat.completion",
            "created": 0,
            "model": "local-test",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": json.dumps({"sentence": LANGUAGES[language]}, ensure_ascii=False),
                },
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 20,
                "total_tokens": 120,
                "prompt_cache_hit_tokens": 40,
                "prompt_tokens_details": {"cached_tokens": 40},
            },
        }, ensure_ascii=False).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        try:
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass
        finally:
            with owner.changed:
                owner.responses.append(call_number)
                owner.changed.notify_all()

    def log_message(self, *_):
        pass


class MockServer(http.server.ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, address):
        super().__init__(address, MockTranslation)
        self.calls = []
        self.responses = []
        self.lock = threading.Lock()
        self.changed = threading.Condition(self.lock)
        self.release = threading.Event()
        self.release.set()

    def reset_calls(self):
        with self.lock:
            self.calls.clear()
            self.responses.clear()

    def snapshot(self):
        with self.lock:
            return list(self.calls)

    def wait_for_calls(self, count, timeout):
        deadline = time.monotonic() + timeout
        with self.changed:
            while len(self.calls) < count:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return False
                self.changed.wait(remaining)
            return True

    def wait_for_responses(self, count, timeout):
        deadline = time.monotonic() + timeout
        with self.changed:
            while len(self.responses) < count:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return False
                self.changed.wait(remaining)
            return True


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def read_exact(pipe, size):
    result = bytearray()
    while len(result) < size:
        chunk = pipe.read(size - len(result))
        if not chunk:
            raise RuntimeError("named pipe closed")
        result.extend(chunk)
    return bytes(result)


def server_pid(pipe):
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    get_pid = kernel32.GetNamedPipeServerProcessId
    get_pid.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.ULONG)]
    get_pid.restype = wintypes.BOOL
    pid = wintypes.ULONG()
    handle = wintypes.HANDLE(msvcrt.get_osfhandle(pipe.fileno()))
    if not get_pid(handle, ctypes.byref(pid)):
        raise ctypes.WinError(ctypes.get_last_error())
    return pid.value


def pipe_exchange(pipe, message):
    data = json.dumps(message, ensure_ascii=False).encode("utf-8")
    pipe.write(struct.pack("<I", len(data)) + data)
    size, = struct.unpack("<I", read_exact(pipe, 4))
    return json.loads(read_exact(pipe, size))


def pipe_send(pipe, message):
    data = json.dumps(message, ensure_ascii=False).encode("utf-8")
    pipe.write(struct.pack("<I", len(data)) + data)


def protocol_version():
    source = (REPO / "crates/qingjian-platform/src/protocol/mod.rs").read_text(encoding="utf-8")
    match = re.search(r"PROTOCOL_VERSION: u32 = (\d+)", source)
    if not match:
        raise RuntimeError("could not read protocol version from workspace source")
    return int(match.group(1))


def make_stage(binary_dir, path):
    server = binary_dir / "ctrllingo-server.exe"
    if not server.is_file():
        raise FileNotFoundError(f"binary directory has no ctrllingo-server.exe: {binary_dir}")
    path.mkdir(parents=True)
    shutil.copy2(server, path / server.name)
    for optional in ("ctrllingo-settings.exe", "qingjian_tsf.dll"):
        source = binary_dir / optional
        if source.is_file():
            shutil.copy2(source, path / optional)
    for name in ("data", "assets"):
        source = binary_dir / name
        if not source.is_dir():
            source = REPO / name
        if source.is_dir():
            shutil.copytree(source, path / name, dirs_exist_ok=True)


def write_config(user_root, port, language):
    config_dir = user_root / "roaming" / "CtrlLingo"
    config_dir.mkdir(parents=True, exist_ok=True)
    config = (REPO / "apps/windows/installer/default-config.toml").read_text(encoding="utf-8")
    config = config.replace('learning_language = "en"', f'learning_language = "{language}"', 1)
    config = config.replace(
        "[predict]\nenabled = false",
        f'[predict]\nbase_url = "http://127.0.0.1:{port}/v1"\nmodel = "local-test"\nenabled = true',
        1,
    )
    config = config.replace("[status_bar]\nenabled = true", "[status_bar]\nenabled = false", 1)
    if 'base_url = "http://127.0.0.1:' not in config:
        raise RuntimeError("could not enable the local mock provider in the default config")
    (config_dir / "config.toml").write_text(config, encoding="utf-8")
    (config_dir / "cloud-controls-v3.migrated").write_text("3\n", encoding="utf-8")


def environment_for(user_root, stage, temp_root, language):
    env = dict(os.environ)
    env.update({
        "APPDATA": str(user_root / "roaming"),
        "LOCALAPPDATA": str(user_root / "local"),
        "TMP": str(temp_root),
        "TEMP": str(temp_root),
        "NO_PROXY": "localhost,127.0.0.1,::1",
        "no_proxy": "localhost,127.0.0.1,::1",
        "CTRLLINGO_API_KEY": "local-test-key",
        "RUST_BACKTRACE": "1",
        "RUST_LOG": "debug",
    })
    for proxy in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"):
        env.pop(proxy, None)
    generated = stage / "data" / "generated"
    dictionary = generated / "dict.qj"
    glossary = generated / f"glossary-{language}.qj"
    if dictionary.is_file():
        env["QINGJIAN_DICT"] = str(dictionary)
    if glossary.is_file():
        env["QINGJIAN_GLOSSARY"] = str(glossary)
    return env


def candidates(frame):
    return frame.get("candidates", {}).get("items", [])


def preedit(frame):
    return "".join(segment.get("text", "") for segment in frame.get("preedit", []))


def send_key(pipe, virtual_key, character=None, ctrl=False):
    return pipe_exchange(pipe, {"Key": {"session": SESSION, "event": {
        "virtual_key": virtual_key,
        "character": character,
        "modifiers": {
            "ctrl": ctrl, "alt": False, "shift": False, "win": False,
            "caps": False, "english_mode": False,
        },
    }}})["KeyResult"]


def send_text(pipe, text):
    result = None
    for character in text:
        result = send_key(pipe, ord(character.upper()), character)
    return result


def poll(pipe):
    return pipe_exchange(pipe, {"Poll": {"session": SESSION}}).get("Update", {})


def wait_for_translation(pipe, expected, timeout=10):
    deadline = time.monotonic() + timeout
    last_frame = {}
    while time.monotonic() < deadline:
        update = poll(pipe)
        last_frame = update.get("frame") or {}
        if any(candidate.get("text") == expected for candidate in candidates(last_frame)):
            return last_frame
        time.sleep(0.08)
    raise AssertionError(f"mock translation did not appear: {expected!r}; last frame={last_frame!r}")


def select_without_local_translation(pipe, frame):
    items = candidates(frame)
    selected_frame = frame
    for index, candidate in enumerate(items):
        annotation = candidate.get("translation")
        senses = annotation.get("senses", []) if annotation else []
        if not any(sense.get("text", "").strip() for sense in senses):
            for _ in range(index):
                selected_frame = send_key(pipe, 0x28)["frame"]
            current = candidates(selected_frame)
            return current[selected_frame["highlight"]].get("text")
    raise AssertionError("this pinyin has no visible candidate without a local translation")


def start_server(stage, user_root, temp_root, language, hold_seconds):
    process = subprocess.Popen(
        [str(stage / "ctrllingo-server.exe")],
        cwd=stage,
        env=environment_for(user_root, stage, temp_root, language),
        creationflags=subprocess.CREATE_NO_WINDOW,
        stdout=open(user_root / "server-output.log", "wb"),
        stderr=subprocess.STDOUT,
    )
    watchdog = threading.Timer(
        WATCHDOG_BASE_SECONDS + hold_seconds,
        lambda: process.kill() if process.poll() is None else None,
    )
    watchdog.daemon = True
    watchdog.start()
    pipe = None
    try:
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            try:
                pipe = open(PIPE_NAME, "r+b", buffering=0)
                break
            except OSError:
                if process.poll() is not None:
                    raise RuntimeError(f"spawned server exited ({process.returncode})")
                time.sleep(0.05)
        if pipe is None:
            raise TimeoutError("spawned server named pipe did not start")
        actual_pid = server_pid(pipe)
        if actual_pid != process.pid:
            raise RuntimeError(
                f"named pipe belongs to another process (pid {actual_pid}); no message was sent"
            )
        opened = pipe_exchange(pipe, {"OpenSession": {
            "session": SESSION,
            "app": "notepad.exe",
            "protocol": protocol_version(),
        }})
        if "SessionOpened" not in opened:
            raise RuntimeError(f"session did not open: {opened!r}")
        return process, watchdog, pipe
    except Exception:
        if pipe is not None:
            pipe.close()
        watchdog.cancel()
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        raise


def wait_mock_call(mock, expected_count, language, expected_text=None):
    if not mock.wait_for_calls(expected_count, 8):
        raise TimeoutError("Ctrl did not reach the local HTTP mock")
    call = mock.snapshot()[expected_count - 1]
    require(call["target_language"] == language, f"wrong target language: {call!r}")
    if expected_text is not None:
        require(call["text"] == expected_text, f"wrong source text: {call!r}")
    return call


def commit_translation(pipe, translation):
    frame = wait_for_translation(pipe, translation)
    result = send_key(pipe, 0x20, " ")
    require(result.get("commit") == translation, f"Space did not commit translation: {result!r}")
    return frame


def capture_window_png(window, output):
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    gdi32 = ctypes.WinDLL("gdi32", use_last_error=True)
    user32.GetWindowDC.argtypes = [wintypes.HWND]
    user32.GetWindowDC.restype = wintypes.HDC
    user32.ReleaseDC.argtypes = [wintypes.HWND, wintypes.HDC]
    user32.PrintWindow.argtypes = [wintypes.HWND, wintypes.HDC, wintypes.UINT]
    user32.PrintWindow.restype = wintypes.BOOL
    user32.SendMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
    user32.SendMessageW.restype = ctypes.c_ssize_t
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    gdi32.CreateCompatibleDC.argtypes = [wintypes.HDC]
    gdi32.CreateCompatibleDC.restype = wintypes.HDC
    gdi32.CreateCompatibleBitmap.argtypes = [wintypes.HDC, ctypes.c_int, ctypes.c_int]
    gdi32.CreateCompatibleBitmap.restype = wintypes.HBITMAP
    gdi32.SelectObject.argtypes = [wintypes.HDC, wintypes.HGDIOBJ]
    gdi32.SelectObject.restype = wintypes.HGDIOBJ
    gdi32.DeleteObject.argtypes = [wintypes.HGDIOBJ]
    gdi32.DeleteDC.argtypes = [wintypes.HDC]

    class BitmapInfo(ctypes.Structure):
        _fields_ = [
            ("size", wintypes.DWORD), ("width", wintypes.LONG), ("height", wintypes.LONG),
            ("planes", wintypes.WORD), ("bit_count", wintypes.WORD),
            ("compression", wintypes.DWORD), ("size_image", wintypes.DWORD),
            ("x", wintypes.LONG), ("y", wintypes.LONG), ("used", wintypes.DWORD),
            ("important", wintypes.DWORD),
        ]

    gdi32.GetDIBits.argtypes = [
        wintypes.HDC, wintypes.HBITMAP, wintypes.UINT, wintypes.UINT,
        ctypes.c_void_p, ctypes.POINTER(BitmapInfo), wintypes.UINT,
    ]
    rect = wintypes.RECT()
    if not user32.GetWindowRect(window, ctypes.byref(rect)):
        raise ctypes.WinError(ctypes.get_last_error())
    width, height = rect.right - rect.left, rect.bottom - rect.top
    if width <= 0 or height <= 0:
        raise RuntimeError("usage panel has an empty window rectangle")
    window_dc = user32.GetWindowDC(window)
    memory_dc = gdi32.CreateCompatibleDC(window_dc)
    bitmap = gdi32.CreateCompatibleBitmap(window_dc, width, height)
    previous = gdi32.SelectObject(memory_dc, bitmap)
    try:
        if not user32.PrintWindow(window, memory_dc, 2):
            raise ctypes.WinError(ctypes.get_last_error())
        # The usage card handles WM_PRINTCLIENT directly; request a completed paint before reading pixels.
        user32.SendMessageW(window, 0x0318, memory_dc, 0x00000004)
        header = BitmapInfo(ctypes.sizeof(BitmapInfo), width, -height, 1, 32, 0, 0, 0, 0, 0, 0)
        pixels = ctypes.create_string_buffer(width * height * 4)
        if not gdi32.GetDIBits(memory_dc, bitmap, 0, height, pixels, ctypes.byref(header), 0):
            raise ctypes.WinError(ctypes.get_last_error())
        rgb = bytearray(width * height * 3)
        rgb[0::3], rgb[1::3], rgb[2::3] = pixels.raw[2::4], pixels.raw[1::4], pixels.raw[0::4]
        require(min(rgb) < 150 and max(rgb) > 240, "native usage panel screenshot is blank")

        def chunk(kind, data):
            return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

        rows = b"".join(b"\0" + rgb[y * width * 3:(y + 1) * width * 3] for y in range(height))
        png = (b"\x89PNG\r\n\x1a\n" +
               chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) +
               chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(png)
    finally:
        gdi32.SelectObject(memory_dc, previous)
        gdi32.DeleteObject(bitmap)
        gdi32.DeleteDC(memory_dc)
        user32.ReleaseDC(window, window_dc)
    return {"path": str(output), "width": width, "height": height}


def open_usage_panel(pipe, process, hold_seconds, screenshot_path):
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    user32.GetForegroundWindow.restype = wintypes.HWND
    foreground_before = user32.GetForegroundWindow()
    pipe_send(pipe, {"Indicator": {"session": SESSION, "command": {
        "OpenUsagePanel": {"x": 120, "y": 120},
    }}})
    sync = pipe_exchange(pipe, {"SyncMode": {"session": SESSION}})
    indicator = sync.get("ModeSync", {}).get("indicator", {})
    balance = indicator.get("balance_summary", "")
    require("不支持" in balance, f"custom mock provider was assigned an unsupported balance status: {balance!r}")
    find_window = user32.FindWindowW
    find_window.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR]
    find_window.restype = wintypes.HWND
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.IsWindowVisible.restype = wintypes.BOOL
    get_window_pid = user32.GetWindowThreadProcessId
    get_window_pid.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    get_window_pid.restype = wintypes.DWORD
    deadline = time.monotonic() + 6
    window = None
    while time.monotonic() < deadline:
        candidate = find_window("CtrlLingoUsageCosts", None)
        if candidate:
            owner_pid = wintypes.DWORD()
            get_window_pid(candidate, ctypes.byref(owner_pid))
            if owner_pid.value == process.pid and user32.IsWindowVisible(candidate):
                window = candidate
                break
        if process.poll() is not None:
            raise RuntimeError("server exited before the native usage panel appeared")
        time.sleep(0.05)
    require(window, "native CtrlLingoUsageCosts window did not open")
    foreground_after = user32.GetForegroundWindow()
    require(foreground_after == foreground_before,
            f"opening the usage panel changed foreground window: {foreground_before} -> {foreground_after}")
    time.sleep(0.6)
    require(process.poll() is None, f"server exited before capture: {process.returncode}")
    screenshot = capture_window_png(window, screenshot_path)
    require(process.poll() is None, f"server exited after capture: {process.returncode}")
    send_message = user32.SendMessageW
    send_message.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
    send_message.restype = wintypes.LPARAM
    send_message(window, 0x0204, 0, 0)
    require(not user32.IsWindowVisible(window), "usage card did not hide")
    tray = find_window("CtrlLingoTrayMessageWindow", None)
    require(tray, "native tray message window not found")
    owner_pid = wintypes.DWORD()
    get_window_pid(tray, ctypes.byref(owner_pid))
    require(owner_pid.value == process.pid, "tray window belongs to another process")
    send_message(tray, 0x8000 + 0x71, 1, 0x0205)
    deadline = time.monotonic() + 3
    while not user32.IsWindowVisible(window) and time.monotonic() < deadline:
        time.sleep(0.05)
    require(user32.IsWindowVisible(window), "tray right-click callback did not reopen the card")
    require(user32.GetForegroundWindow() == foreground_before, "tray callback changed input focus")
    if hold_seconds:
        time.sleep(hold_seconds)
    return {
        "tray_right_click_callback_reopens": True,
        "window_found": True,
        "visible": True,
        "belongs_to_server_process": True,
        "foreground_unchanged": True,
        "screenshot": screenshot,
        "balance_summary": balance,
        "hold_seconds": hold_seconds,
    }


def run_language(stage, run_id, temp_root, output_dir, mock, language, hold_seconds):
    label = f"{language}-{uuid.uuid4().hex[:8]}"
    user_root = TOOLS / f"v3-smoke-user-{run_id}-{label}"
    write_config(user_root, mock.server_port, language)
    mock.reset_calls()
    process, watchdog, pipe = start_server(stage, user_root, temp_root, language, hold_seconds)
    result = {"language": language, "stage": str(stage), "user_root": str(user_root)}
    try:
        send_text(pipe, "nihao")
        for _ in range(4):
            poll(pipe)
            time.sleep(0.08)
        require(not mock.snapshot(), "ordinary typing or stopping made an HTTP request")
        send_key(pipe, 0x1B)

        typed = send_text(pipe, "wojintianxiangqugongyuan")
        selected_text = select_without_local_translation(pipe, typed["frame"])
        time.sleep(0.45)
        for _ in range(3):
            poll(pipe)
        require(not mock.snapshot(), "ordinary composition made an HTTP request before Ctrl")

        # First request is held while editing, so its late response must not enter the new composition.
        mock.release.clear()
        send_key(pipe, 0x100)
        stale = wait_mock_call(mock, 1, language, selected_text)
        waiting = send_key(pipe, 0x20, " ")
        require(waiting.get("commit") in (None, ""), "Space committed while the provider was holding its reply")
        require(preedit(waiting["frame"]).replace("'", "").replace(" ", "") == "wojintianxiangqugongyuan",
                "waiting Space lost the pinyin composition")
        edited = send_key(pipe, ord("Z"), "z")
        require("z" in preedit(edited["frame"]).lower(), "editing did not update the composition")
        mock.release.set()
        require(mock.wait_for_responses(1, 5), "mock did not finish the edited request")
        stale_deadline = time.monotonic() + 3
        stale_frame = {}
        stale_translation_seen = False
        while time.monotonic() < stale_deadline:
            stale_update = poll(pipe)
            stale_frame = stale_update.get("frame") or {}
            stale_translation_seen |= any(
                candidate.get("text") == LANGUAGES[language]
                for candidate in candidates(stale_frame)
            )
            time.sleep(0.08)
        require(not stale_translation_seen,
                "late result contaminated the edited composition")
        send_key(pipe, 0x1B)

        # The next request is allowed through and its real Unicode result must be displayable and committable.
        typed = send_text(pipe, "xiangkaifa")
        selected_again = select_without_local_translation(pipe, typed["frame"])
        require(selected_again != selected_text, "stale and normal scenarios must use different source text")
        selected_text = selected_again
        mock.release.clear()
        send_key(pipe, 0x100)
        normal = wait_mock_call(mock, 2, language, selected_text)
        waiting = send_key(pipe, 0x20, " ")
        require(waiting.get("commit") in (None, ""), "Space committed before the mock response was released")
        pending_preedit = preedit(waiting["frame"])
        calls_before_panel = len(mock.snapshot())
        screenshot_path = output_dir / f"v3-smoke-{run_id}-{language}-usage.png"
        usage_panel = open_usage_panel(pipe, process, hold_seconds, screenshot_path)
        require(len(mock.snapshot()) == calls_before_panel, "opening the usage panel made an HTTP model request")
        pending_update = poll(pipe)
        require(preedit(pending_update.get("frame") or {}) == pending_preedit,
                "opening the usage panel changed the pending pinyin composition")
        waiting_again = send_key(pipe, 0x20, " ")
        require(waiting_again.get("commit") in (None, ""),
                "Space committed while the translation was still held after opening the usage panel")
        require(preedit(waiting_again["frame"]) == pending_preedit,
                "waiting Space after opening the usage panel changed the pinyin composition")
        mock.release.set()
        frame = commit_translation(pipe, LANGUAGES[language])
        require(any(candidate.get("text") == LANGUAGES[language] for candidate in candidates(frame)),
                "Unicode translation was not visible in the native protocol frame")

        # Re-enter the same Chinese source: the language-specific cached result should avoid another POST.
        typed = send_text(pipe, "xiangkaifa")
        select_without_local_translation(pipe, typed["frame"])
        before_cache = len(mock.snapshot())
        send_key(pipe, 0x100)
        cache_deadline = time.monotonic() + 1.5
        cache_frame = {}
        while time.monotonic() < cache_deadline:
            update = poll(pipe)
            cache_frame = update.get("frame") or {}
            if any(candidate.get("text") == LANGUAGES[language] for candidate in candidates(cache_frame)):
                break
            time.sleep(0.05)
        require(len(mock.snapshot()) == before_cache, "repeating Ctrl for the same text missed the translation cache")
        commit = send_key(pipe, 0x20, " ")
        require(commit.get("commit") == LANGUAGES[language], "cached translation did not commit")
        result.update({
            "request": normal,
            "stale_request": stale,
            "translation": LANGUAGES[language],
            "http_request_count": len(mock.snapshot()),
            "cache_reused": True,
        })

        ledger_path = user_root / "roaming" / "CtrlLingo" / "api-cost-ledger.json"
        deadline = time.monotonic() + 5
        ledger = None
        while time.monotonic() < deadline:
            try:
                ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
                if ledger.get("attempts"):
                    break
            except (OSError, json.JSONDecodeError):
                pass
            time.sleep(0.05)
        require(ledger is not None, "cost ledger was not written")
        attempts = ledger.get("attempts", [])
        require(len(attempts) == 2, f"expected two actual HTTP attempts, got {len(attempts)}")
        for attempt in attempts:
            require(attempt.get("purpose") == "translate", f"wrong ledger purpose: {attempt!r}")
            require(attempt.get("input_tokens") == 100, f"wrong input token count: {attempt!r}")
            require(attempt.get("output_tokens") == 20, f"wrong output token count: {attempt!r}")
            require(attempt.get("cached_input_tokens") == 40, f"wrong cache token count: {attempt!r}")
        result["ledger"] = {
            "attempts": len(attempts),
            "purpose": [attempt.get("purpose") for attempt in attempts],
            "input_tokens": [attempt.get("input_tokens") for attempt in attempts],
            "output_tokens": [attempt.get("output_tokens") for attempt in attempts],
            "cached_input_tokens": [attempt.get("cached_input_tokens") for attempt in attempts],
        }
        time.sleep(1.5)
        desktop = ctypes.WinDLL("user32", use_last_error=True)
        desktop.FindWindowW.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR]
        desktop.FindWindowW.restype = wintypes.HWND
        card = desktop.FindWindowW("CtrlLingoUsageCosts", None)
        usage_panel["completed_screenshot"] = capture_window_png(
            card, output_dir / f"v3-smoke-{run_id}-{language}-completed.png",
        )
        result["usage_panel"] = usage_panel
        require(len(mock.snapshot()) == 2, "opening usage panel made an HTTP model request")
        return result
    finally:
        time.sleep(0.8)
        print(f"server pid={process.pid} exit_before_cleanup={process.poll()} user_root={user_root}", flush=True)
        mock.release.set()
        pipe.close()
        watchdog.cancel()
        if process.poll() is None:
            process.kill()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass


def main():
    parser = argparse.ArgumentParser(description="Local multilingual Server and usage-panel smoke test")
    parser.add_argument("--binary-dir", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--hold-ui", type=float, default=0, metavar="SECONDS")
    args = parser.parse_args()
    if args.hold_ui < 0:
        raise ValueError("--hold-ui must be zero or greater")
    desktop = ctypes.WinDLL("user32", use_last_error=True)
    desktop.SetProcessDpiAwarenessContext.argtypes = [ctypes.c_void_p]
    desktop.SetProcessDpiAwarenessContext.restype = wintypes.BOOL
    desktop.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4))
    binary_dir = args.binary_dir.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    run_id = uuid.uuid4().hex[:10]
    temp_root = TOOLS / "test-temp" / f"v3-smoke-{run_id}"
    temp_root.mkdir(parents=True, exist_ok=True)
    stage = TOOLS / f"v3-smoke-stage-{run_id}"
    make_stage(binary_dir, stage)

    mock = MockServer(("127.0.0.1", 0))
    threading.Thread(target=mock.serve_forever, daemon=True).start()
    results = []
    try:
        for index, language in enumerate(LANGUAGES):
            hold_seconds = args.hold_ui if index == len(LANGUAGES) - 1 else 0
            results.append(run_language(
                stage, run_id, temp_root, output_dir, mock, language, hold_seconds,
            ))
        document = {
            "ok": True,
            "run_id": run_id,
            "languages": results,
            "registered_ime": False,
            "used_external_network": False,
            "real_tsf_verified": False,
            "balance_source": "custom local provider; official balance unsupported",
            "hold_ui_seconds": args.hold_ui,
        }
    except Exception as error:
        document = {
            "ok": False,
            "run_id": run_id,
            "completed_languages": results,
            "error": str(error),
            "registered_ime": False,
            "used_external_network": False,
            "real_tsf_verified": False,
        }
        raise
    finally:
        mock.shutdown()
        mock.server_close()
        (output_dir / f"v3-smoke-{run_id}.json").write_text(
            json.dumps(document, ensure_ascii=False, indent=2), encoding="utf-8",
        )
        print(json.dumps(document, ensure_ascii=False))


if __name__ == "__main__":
    if os.name != "nt":
        print("This smoke test requires Windows.", file=sys.stderr)
        raise SystemExit(2)
    main()
