"""Message-level native UI checks using an isolated staged Qingjian Server."""

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import importlib.util
import json
from pathlib import Path
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
RELEASE_SERVER = REPO / "target" / "release" / "ctrllingo-server.exe"
TIMEOUT_SECONDS = 150

SPEC = importlib.util.spec_from_file_location("v3_smoke", Path(__file__).with_name("smoke.py"))
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load .tools/smoke.py")
V3 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(V3)


class PROCESSENTRY32W(ctypes.Structure):
    _fields_ = [
        ("dwSize", wintypes.DWORD),
        ("cntUsage", wintypes.DWORD),
        ("th32ProcessID", wintypes.DWORD),
        ("th32DefaultHeapID", ctypes.c_size_t),
        ("th32ModuleID", wintypes.DWORD),
        ("cntThreads", wintypes.DWORD),
        ("th32ParentProcessID", wintypes.DWORD),
        ("pcPriClassBase", wintypes.LONG),
        ("dwFlags", wintypes.DWORD),
        ("szExeFile", wintypes.WCHAR * 260),
    ]


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def process_snapshot():
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel32.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    kernel32.Process32FirstW.argtypes = [wintypes.HANDLE, ctypes.POINTER(PROCESSENTRY32W)]
    kernel32.Process32FirstW.restype = wintypes.BOOL
    kernel32.Process32NextW.argtypes = [wintypes.HANDLE, ctypes.POINTER(PROCESSENTRY32W)]
    kernel32.Process32NextW.restype = wintypes.BOOL
    kernel32.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel32.OpenProcess.restype = wintypes.HANDLE
    kernel32.QueryFullProcessImageNameW.argtypes = [
        wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD),
    ]
    kernel32.QueryFullProcessImageNameW.restype = wintypes.BOOL
    kernel32.CloseHandle.argtypes = [wintypes.HANDLE]

    snapshot = kernel32.CreateToolhelp32Snapshot(0x00000002, 0)
    if snapshot == wintypes.HANDLE(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    records = []
    try:
        entry = PROCESSENTRY32W()
        entry.dwSize = ctypes.sizeof(entry)
        more = kernel32.Process32FirstW(snapshot, ctypes.byref(entry))
        while more:
            pid = int(entry.th32ProcessID)
            handle = kernel32.OpenProcess(0x1000, False, pid)
            image = ""
            if handle:
                try:
                    buffer = ctypes.create_unicode_buffer(32768)
                    length = wintypes.DWORD(len(buffer))
                    if kernel32.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(length)):
                        image = buffer.value
                finally:
                    kernel32.CloseHandle(handle)
            records.append({
                "pid": pid,
                "parent_pid": int(entry.th32ParentProcessID),
                "name": entry.szExeFile,
                "path": image,
            })
            more = kernel32.Process32NextW(snapshot, ctypes.byref(entry))
    finally:
        kernel32.CloseHandle(snapshot)
    return records


def existing_qingjian_processes():
    return [
        row for row in process_snapshot()
        if "ctrllingo" in row["name"].lower() or "ctrllingo" in Path(row["path"]).name.lower()
    ]


def window_api():
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    user32.FindWindowW.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR]
    user32.FindWindowW.restype = wintypes.HWND
    user32.IsWindow.argtypes = [wintypes.HWND]
    user32.IsWindow.restype = wintypes.BOOL
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.IsWindowVisible.restype = wintypes.BOOL
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    user32.GetWindowRect.restype = wintypes.BOOL
    user32.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    user32.GetClientRect.restype = wintypes.BOOL
    user32.PostMessageW.argtypes = [
        wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM,
    ]
    user32.PostMessageW.restype = wintypes.BOOL
    user32.GetWindowTextLengthW.argtypes = [wintypes.HWND]
    user32.GetWindowTextLengthW.restype = ctypes.c_int
    user32.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user32.GetWindowTextW.restype = ctypes.c_int
    user32.GetClassNameW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user32.GetClassNameW.restype = ctypes.c_int
    return user32


def window_pid(user32, hwnd):
    pid = wintypes.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
    return pid.value


def wait_until(deadline, seconds, predicate, description):
    end = min(deadline, time.monotonic() + seconds)
    while time.monotonic() < end:
        value = predicate()
        if value:
            return value
        time.sleep(0.05)
    raise TimeoutError(f"timed out waiting for {description}")


def bounded_call(deadline, function, *args, **kwargs):
    remaining = deadline - time.monotonic()
    require(remaining > 0, "overall 150-second timeout expired")
    completed = threading.Event()
    result = []

    def invoke():
        try:
            result.append((True, function(*args, **kwargs)))
        except BaseException as error:
            result.append((False, error))
        finally:
            completed.set()

    threading.Thread(target=invoke, daemon=True).start()
    if not completed.wait(remaining):
        raise TimeoutError("overall 150-second timeout expired during blocking operation")
    succeeded, value = result[0]
    if not succeeded:
        raise value
    return value


def post(user32, hwnd, message, wparam=0, lparam=0, description="PostMessage"):
    if not user32.PostMessageW(hwnd, message, wparam, lparam):
        raise ctypes.WinError(ctypes.get_last_error(), description)


def post_click(user32, hwnd, logical_x, logical_y):
    client = wintypes.RECT()
    if not user32.GetClientRect(hwnd, ctypes.byref(client)):
        raise ctypes.WinError(ctypes.get_last_error())
    width = client.right - client.left
    require(width > 0, "usage panel client area has zero width")
    ratio = width / 360.0
    x = round(logical_x * ratio)
    y = round(logical_y * ratio)
    packed = ((y & 0xFFFF) << 16) | (x & 0xFFFF)
    post(user32, hwnd, 0x0201, 1, packed, "posting WM_LBUTTONDOWN")


def owned_visible_windows(user32, pid):
    windows = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    def collect(hwnd, _):
        if user32.IsWindowVisible(hwnd) and window_pid(user32, hwnd) == pid:
            rect = wintypes.RECT()
            if user32.GetWindowRect(hwnd, ctypes.byref(rect)):
                title_size = user32.GetWindowTextLengthW(hwnd)
                title_buffer = ctypes.create_unicode_buffer(title_size + 1)
                user32.GetWindowTextW(hwnd, title_buffer, len(title_buffer))
                windows.append({
                    "hwnd": hwnd,
                    "rect": (rect.left, rect.top, rect.right, rect.bottom),
                    "title": title_buffer.value,
                })
        return True

    callback = callback_type(collect)
    if not user32.EnumWindows(callback, 0):
        error = ctypes.get_last_error()
        if error:
            raise ctypes.WinError(error)
    return windows


def png_pixel(path, x, y):
    content = path.read_bytes()
    if content[:8] != b"\x89PNG\r\n\x1a\n":
        raise AssertionError(f"invalid PNG signature: {path}")
    offset = 8
    compressed = bytearray()
    width = height = None
    while offset < len(content):
        size = struct.unpack_from(">I", content, offset)[0]
        kind = content[offset + 4:offset + 8]
        chunk = content[offset + 8:offset + 8 + size]
        if kind == b"IHDR":
            width, height, bit_depth, color_type, _, _, _ = struct.unpack(">IIBBBBB", chunk)
            require(bit_depth == 8 and color_type == 2, "unexpected screenshot PNG format")
        elif kind == b"IDAT":
            compressed.extend(chunk)
        elif kind == b"IEND":
            break
        offset += size + 12
    require(width is not None and height is not None, "screenshot has no IHDR")
    require(0 <= x < width and 0 <= y < height, "sample pixel is outside screenshot")
    raw = zlib.decompress(compressed)
    stride = width * 3 + 1
    row = raw[y * stride:(y + 1) * stride]
    require(row[0] == 0, "unexpected PNG row filter")
    start = 1 + x * 3
    return tuple(row[start:start + 3])


def wait_selected_tab(user32, window, selected_x, scratch_path, deadline):
    def selected():
        capture = bounded_call(deadline, V3.capture_window_png, window, scratch_path)
        ratio = capture["width"] / 360.0
        x = round(selected_x * ratio)
        y = round(23 * ratio)
        red, green, blue = png_pixel(scratch_path, x, y)
        return (210 < red < 240 and green - red > 10 and blue > 230)

    return wait_until(deadline, 8, selected, f"period tab at logical x={selected_x} to highlight")


def find_owned_class_window(user32, class_name, owner_pid, visible=True, title=None):
    matches = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    def collect(hwnd, _):
        if window_pid(user32, hwnd) != owner_pid:
            return True
        name = ctypes.create_unicode_buffer(256)
        user32.GetClassNameW(hwnd, name, len(name))
        if name.value != class_name or bool(user32.IsWindowVisible(hwnd)) != visible:
            return True
        if title is not None:
            text = ctypes.create_unicode_buffer(user32.GetWindowTextLengthW(hwnd) + 1)
            user32.GetWindowTextW(hwnd, text, len(text))
            if text.value != title:
                return True
        matches.append(hwnd)
        return True

    user32.EnumWindows(callback_type(collect), 0)
    return matches[0] if matches else None


def find_usage_window(user32, server_pid, visible):
    return find_owned_class_window(user32, "CtrlLingoUsageCosts", server_pid, visible)


def find_popup(user32, class_name, owner_pid):
    return find_owned_class_window(user32, class_name, owner_pid)


def find_named_window(user32, class_name, title, owner_pid):
    return find_owned_class_window(user32, class_name, owner_pid, title=title)


def settings_process(stage, server_pid, before_pids):
    expected = (stage / "ctrllingo-settings.exe").resolve()
    for row in process_snapshot():
        if Path(row["name"]).name.lower() != expected.name.lower():
            continue
        try:
            actual = Path(row["path"]).resolve()
        except (OSError, RuntimeError):
            continue
        if actual != expected:
            continue
        if row["parent_pid"] == server_pid or row["pid"] not in before_pids:
            return row
    return None


def terminate_owned_settings(pid, expected_path):
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel32.OpenProcess.restype = wintypes.HANDLE
    kernel32.QueryFullProcessImageNameW.argtypes = [
        wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD),
    ]
    kernel32.QueryFullProcessImageNameW.restype = wintypes.BOOL
    kernel32.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
    kernel32.TerminateProcess.restype = wintypes.BOOL
    kernel32.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel32.WaitForSingleObject.restype = wintypes.DWORD
    kernel32.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel32.OpenProcess(0x1000 | 0x0001 | 0x00100000, False, pid)
    if not handle:
        return False
    try:
        buffer = ctypes.create_unicode_buffer(32768)
        length = wintypes.DWORD(len(buffer))
        if not kernel32.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(length)):
            return False
        if Path(buffer.value).resolve() != expected_path.resolve():
            return False
        if not kernel32.TerminateProcess(handle, 1):
            return False
        kernel32.WaitForSingleObject(handle, 5000)
        return True
    finally:
        kernel32.CloseHandle(handle)


def make_report(output_dir, run_id, report):
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / f"v4-ui-interactions-{run_id}.json"
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    return path


def run(output_dir, stage):
    run_id = uuid.uuid4().hex[:10]
    report = {
        "ok": False,
        "executed": False,
        "run_id": run_id,
        "stage": str(stage),
        "validation_scope": {
            "interaction_level": "PostMessage message-level only",
            "registered_ime": False,
            "real_tsf_verified": False,
            "real_account_verified": False,
            "physical_mouse_or_keyboard_verified": False,
            "used_external_network": False,
        },
        "steps": [],
    }
    user_root = TOOLS / "v4-ui-user" / run_id
    temp_root = TOOLS / "test-temp" / f"v4-ui-{run_id}"
    process = watchdog = pipe = mock = None
    settings_pid = None
    deadline = time.monotonic() + TIMEOUT_SECONDS
    try:
        require(stage.is_dir(), f"staged directory does not exist: {stage}")
        staged_server = stage / "ctrllingo-server.exe"
        require(staged_server.is_file(), f"staged Server missing: {staged_server}")
        require(RELEASE_SERVER.is_file(), f"release Server missing: {RELEASE_SERVER}")
        stage_hash = digest(staged_server)
        release_hash = digest(RELEASE_SERVER)
        report["server_sha256"] = {
            "stage": stage_hash,
            "release": release_hash,
            "identical": stage_hash == release_hash,
        }
        require(stage_hash == release_hash, "staged Server SHA-256 differs from target/release Server")

        existing = existing_qingjian_processes()
        conflicts = [row for row in existing
                     if row["name"].lower() != "ctrllingo-server.exe"
                     or not row["path"]
                     or Path(row["path"]).resolve() == staged_server.resolve()]
        report["existing_servers_left_running"] = [row for row in existing if row not in conflicts]
        if conflicts:
            report["not_executed_reason"] = {
                "kind": "existing_qingjian_process",
                "processes": conflicts,
            }
            return report

        user32 = window_api()
        user32.SetProcessDpiAwarenessContext.argtypes = [ctypes.c_void_p]
        user32.SetProcessDpiAwarenessContext.restype = wintypes.BOOL
        require(
            user32.SetProcessDpiAwarenessContext(ctypes.c_void_p(-4)),
            "SetProcessDpiAwarenessContext(-4) failed",
        )
        output_dir.mkdir(parents=True, exist_ok=True)
        temp_root.mkdir(parents=True, exist_ok=True)
        user_root.mkdir(parents=True, exist_ok=True)
        mock = V3.MockServer(("127.0.0.1", 0))
        threading.Thread(target=mock.serve_forever, daemon=True).start()
        V3.write_config(user_root, mock.server_port, "en")
        process, watchdog, pipe = V3.start_server(
            stage, user_root, temp_root, "en", 0,
        )
        require(process.poll() is None, "Server exited after launch")
        report["server_pid"] = process.pid
        report["executed"] = True
        report["steps"].append({"name": "staged_server_pid", "ok": True, "pid": process.pid})

        V3.pipe_send(pipe, {"Indicator": {"session": V3.SESSION, "command": {
            "OpenUsagePanel": {"x": 120, "y": 120},
        }}})
        sync = bounded_call(
            deadline, V3.pipe_exchange, pipe, {"SyncMode": {"session": V3.SESSION}},
        )
        balance = sync.get("ModeSync", {}).get("indicator", {}).get("balance_summary", "")
        require("不支持" in balance, f"local mock provider balance state was unexpected: {balance!r}")
        window = wait_until(
            deadline, 8,
            lambda: find_usage_window(user32, process.pid, True),
            "visible CtrlLingoUsageCosts window owned by staged Server",
        )
        report["steps"].append({
            "name": "usage_window_pid_and_visibility", "ok": True,
            "hwnd": int(window), "pid": window_pid(user32, window), "balance_summary": balance,
        })

        for label, logical_x, sample_x in (
            ("today", 242, 220),
            ("cumulative", 294, 272),
        ):
            post_click(user32, window, logical_x, 23)
            screenshot_path = output_dir / f"v4-ui-{run_id}-{label}.png"
            scratch_path = temp_root / f"{label}-poll.png"
            wait_selected_tab(user32, window, sample_x, scratch_path, deadline)
            screenshot = bounded_call(deadline, V3.capture_window_png, window, screenshot_path)
            ratio = screenshot["width"] / 360.0
            pixel = png_pixel(screenshot_path, round(sample_x * ratio), round(23 * ratio))
            require(210 < pixel[0] < 240 and pixel[1] - pixel[0] > 10 and pixel[2] > 230,
                    f"{label} tab did not show the selected mint state: {pixel!r}")
            other_x = 272 if label == "today" else 220
            other_pixel = png_pixel(screenshot_path, round(other_x * ratio), round(23 * ratio))
            require(other_pixel[0] >= 240, f"inactive tab was still highlighted: {other_pixel!r}")
            report["steps"].append({
                "name": f"{label}_coordinate_message_and_screenshot", "ok": True,
                "message": "WM_LBUTTONDOWN via PostMessage",
                "screenshot": screenshot,
                "selected_tab_pixel_rgb": pixel,
                "inactive_tab_pixel_rgb": other_pixel,
            })

        post_click(user32, window, 180, 365)
        menu = wait_until(
            deadline, 6,
            lambda: find_popup(user32, "#32768", process.pid),
            "native #32768 language popup owned by Server",
        )
        report["steps"].append({
            "name": "native_language_popup", "ok": True,
            "hwnd": int(menu), "owner_pid": window_pid(user32, menu),
        })
        post(user32, window, 0x0111, 2, 0, "posting Japanese language WM_COMMAND")
        post(user32, menu, 0x0100, 0x1B, 0, "posting menu Escape")
        config_path = user_root / "roaming" / "CtrlLingo" / "config.toml"
        wait_until(
            deadline, 8,
            lambda: config_path.is_file()
            and 'learning_language = "ja"' in config_path.read_text(encoding="utf-8"),
            "isolated config learning_language=ja",
        )
        wait_until(deadline, 4, lambda: not user32.IsWindowVisible(menu), "language popup to close")
        report["steps"].append({
            "name": "select_japanese_and_persist_isolated_config", "ok": True,
            "message": "WM_COMMAND id=2 and WM_KEYDOWN VK_ESCAPE via PostMessage",
            "config": str(config_path), "language": "ja",
        })

        post_click(user32, window, 90, 405)
        dialog = wait_until(
            deadline, 6,
            lambda: find_named_window(user32, "#32770", "译点·详细记录", process.pid),
            "native detailed-record MessageBox",
        )
        require(window_pid(user32, dialog) == process.pid, "details dialog belongs to another process")
        report["steps"].append({
            "name": "details_dialog_appears", "ok": True,
            "hwnd": int(dialog), "pid": window_pid(user32, dialog),
        })
        post(user32, dialog, 0x0010, 0, 0, "posting WM_CLOSE to details dialog")
        wait_until(deadline, 4, lambda: not user32.IsWindow(dialog), "details dialog to close")
        report["steps"].append({"name": "details_dialog_closed", "ok": True})

        post_click(user32, window, 338, 20)
        wait_until(deadline, 4, lambda: not user32.IsWindowVisible(window), "usage panel to hide")
        tray = find_owned_class_window(user32, "CtrlLingoTrayMessageWindow", process.pid, visible=False)
        require(tray, "Server tray message window not found")
        require(window_pid(user32, tray) == process.pid, "tray callback window belongs to another process")
        post(user32, tray, 0x8071, 1, 0x0205, "posting tray right-button callback")
        wait_until(deadline, 5, lambda: user32.IsWindowVisible(window), "tray callback to reopen usage panel")
        report["steps"].append({
            "name": "close_button_hides_and_tray_callback_reopens", "ok": True,
            "messages": ["WM_LBUTTONDOWN at close coordinates", "WM_APP+0x71 tray WM_RBUTTONUP"],
            "tray_pid": window_pid(user32, tray),
        })

        before_settings = {row["pid"] for row in process_snapshot()}
        post_click(user32, window, 250, 405)
        settings = wait_until(
            deadline, 12,
            lambda: settings_process(stage, process.pid, before_settings),
            "new sibling ctrllingo-settings.exe with exact staged path",
        )
        settings_pid = settings["pid"]
        require(
            settings["parent_pid"] == process.pid or settings_pid not in before_settings,
            "settings process was neither a Server child nor a newly created PID",
        )
        expected_settings = (stage / "ctrllingo-settings.exe").resolve()
        require(Path(settings["path"]).resolve() == expected_settings,
                f"settings executable path did not match staged sibling: {settings['path']!r}")
        settings_windows = wait_until(
            deadline, 18,
            lambda: [row for row in owned_visible_windows(user32, settings_pid)
                     if row["rect"][2] - row["rect"][0] > 100
                     and row["rect"][3] - row["rect"][1] > 100],
            "visible native settings window owned by staged settings process",
        )
        require(any(row["rect"][2] - row["rect"][0] > 100
                    and row["rect"][3] - row["rect"][1] > 100 for row in settings_windows),
                "settings window was too small to be a usable settings page")
        report["steps"].append({
            "name": "settings_process_and_real_window", "ok": True,
            "pid": settings_pid, "parent_pid": settings["parent_pid"],
            "exe": settings["path"],
            "window_titles": [row["title"] for row in settings_windows],
            "window_rects": [row["rect"] for row in settings_windows],
        })
        report["ok"] = True
        return report
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
        return report
    finally:
        if settings_pid is not None:
            terminate_owned_settings(settings_pid, stage / "ctrllingo-settings.exe")
        if pipe is not None:
            try:
                pipe.close()
            except OSError:
                pass
        if watchdog is not None:
            watchdog.cancel()
        if process is not None and process.poll() is None:
            process.kill()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass
        if mock is not None:
            mock.shutdown()
            mock.server_close()


def main():
    parser = argparse.ArgumentParser(description="Isolated message-level usage-panel interactions")
    parser.add_argument(
        "--output-dir", type=Path, default=TOOLS / "v4-ui-interactions-output",
        help="directory for the JSON report and two screenshots",
    )
    parser.add_argument(
        "--stage", type=Path, default=REPO / "target/acceptance-stage",
        help="isolated staged directory containing the Server and Settings executables",
    )
    args = parser.parse_args()
    report = run(args.output_dir.resolve(), args.stage.resolve())
    path = make_report(args.output_dir.resolve(), report["run_id"], report)
    report["report_path"] = str(path)
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if report.get("not_executed_reason"):
        return 2
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
