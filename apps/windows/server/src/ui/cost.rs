//! 常驻隐藏的用量与余额卡片；显示时不激活，避免输入焦点离开当前应用。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush,
    DEFAULT_CHARSET, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, DeleteObject,
    DrawTextW, EndPaint, FF_DONTCARE, FW_BOLD, FillRect, HDC, OUT_TT_PRECIS, PAINTSTRUCT,
    SelectObject, SetBkMode, SetTextColor, TRANSPARENT, VARIABLE_PITCH,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_ESCAPE, VK_LBUTTON, VK_RBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, KillTimer, LoadCursorW, MA_NOACTIVATE, MF_STRING, SW_HIDE, SetTimer, ShowWindow,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_COMMAND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MOUSEACTIVATE,
    WM_PAINT, WM_PRINTCLIENT, WM_RBUTTONDOWN, WM_TIMER, WNDCLASSEXW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{PCWSTR, Result, w};

use super::monitor::{dpi_near, work_area_near};
use super::window_class::WindowClass;
use crate::dispatch::{CostPanelView, StatusEvent};

const CLASS_NAME: PCWSTR = w!("QingjianUsageCosts");
const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 50;
const LOGICAL_WIDTH: i32 = 390;
const LOGICAL_HEIGHT: i32 = 450;
const MARGIN: i32 = 20;
const BLUE: COLORREF = COLORREF(0x00B77A35);
const TEXT: COLORREF = COLORREF(0x00211C18);
const MUTED: COLORREF = COLORREF(0x00746D66);
static CLASS: WindowClass = WindowClass::new();

struct Shared {
    events: Rc<dyn Fn(StatusEvent)>,
    view: RefCell<CostPanelView>,
    total: Cell<bool>,
    visible: Cell<bool>,
    timer_ticks: Cell<u8>,
    left_was_down: Cell<bool>,
    right_was_down: Cell<bool>,
    scale_dpi: Cell<u32>,
    menu_open: Cell<bool>,
}

thread_local! {
    static WINDOWS: RefCell<HashMap<isize, Rc<Shared>>> = RefCell::new(HashMap::new());
}

/// 用量卡片单例；窗口始终存在，平时隐藏。
pub(crate) struct CostWindow {
    hwnd: HWND,
    shared: Rc<Shared>,
}

impl CostWindow {
    pub(crate) fn new(events: Rc<dyn Fn(StatusEvent)>) -> Result<Self> {
        CLASS.ensure(|| WNDCLASSEXW {
            lpfnWndProc: Some(wndproc),
            hInstance: super::module_handle(),
            hCursor: unsafe {
                LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW)
            }
            .unwrap_or_default(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        })?;
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                CLASS_NAME,
                w!("青简·用量与余额"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(super::module_handle()),
                None,
            )?
        };
        let shared = Rc::new(Shared {
            events,
            view: RefCell::new(CostPanelView::default()),
            total: Cell::new(false),
            visible: Cell::new(false),
            timer_ticks: Cell::new(0),
            left_was_down: Cell::new(false),
            right_was_down: Cell::new(false),
            scale_dpi: Cell::new(96),
            menu_open: Cell::new(false),
        });
        WINDOWS.with(|windows| {
            windows.borrow_mut().insert(hwnd.0 as isize, shared.clone());
        });
        Ok(Self { hwnd, shared })
    }

    pub(crate) fn show(&self, view: CostPanelView, point: (i32, i32)) {
        *self.shared.view.borrow_mut() = view;
        self.shared.total.set(false);
        self.shared.visible.set(true);
        self.shared.timer_ticks.set(0);
        self.shared.left_was_down.set(key_down(VK_LBUTTON.0 as i32));
        self.shared
            .right_was_down
            .set(key_down(VK_RBUTTON.0 as i32));
        let anchor = POINT {
            x: point.0,
            y: point.1,
        };
        let work = work_area_near(anchor);
        let dpi = dpi_near(anchor).unwrap_or(96);
        let available_width = work.right.saturating_sub(work.left).max(1);
        let available_height = work.bottom.saturating_sub(work.top).max(1);
        let scale_dpi = dpi
            .min((available_width as u32).saturating_mul(96) / LOGICAL_WIDTH as u32)
            .min((available_height as u32).saturating_mul(96) / LOGICAL_HEIGHT as u32)
            .max(1);
        self.shared.scale_dpi.set(scale_dpi);
        let scale = |value: i32| value * scale_dpi as i32 / 96;
        let width = scale(LOGICAL_WIDTH).max(1);
        let height = scale(LOGICAL_HEIGHT).max(1);
        let margin_x = scale(MARGIN).min((available_width - width).max(0) / 2);
        let margin_y = scale(MARGIN).min((available_height - height).max(0) / 2);
        let left = point
            .0
            .clamp(work.left + margin_x, work.right - width - margin_x);
        let top = point
            .1
            .clamp(work.top + margin_y, work.bottom - height - margin_y);
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                self.hwnd,
                Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                left,
                top,
                width,
                height,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
            );
            let _ = SetTimer(Some(self.hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(self.hwnd), None, true);
        }
        (self.shared.events)(StatusEvent::PollUsageCosts { total: false });
    }

    pub(crate) fn update(&self, view: CostPanelView) {
        if *self.shared.view.borrow() == view {
            return;
        }
        *self.shared.view.borrow_mut() = view;
        if self.shared.visible.get() {
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(self.hwnd), None, true);
            }
        }
    }

    fn hide(&self) {
        hide_window(self.hwnd, &self.shared);
    }
}

impl Drop for CostWindow {
    fn drop(&mut self) {
        self.hide();
        WINDOWS.with(|windows| {
            windows.borrow_mut().remove(&(self.hwnd.0 as isize));
        });
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn key_down(key: i32) -> bool {
    unsafe { GetAsyncKeyState(key) < 0 }
}

fn hide_window(hwnd: HWND, shared: &Shared) {
    shared.menu_open.set(false);
    if shared.visible.replace(false) {
        unsafe {
            let _ = KillTimer(Some(hwnd), TIMER_ID);
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    WINDOWS.with(|windows| {
        let shared = windows.borrow().get(&(hwnd.0 as isize)).cloned();
        let Some(shared) = shared else {
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        };
        match message {
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
            WM_PAINT => {
                paint(hwnd, &shared);
                LRESULT(0)
            }
            WM_PRINTCLIENT => {
                paint_content(HDC(wparam.0 as *mut _), &shared);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let scale_dpi = shared.scale_dpi.get().max(1) as i32;
                let p = POINT {
                    x: (lparam.0 as u16 as i16) as i32 * 96 / scale_dpi,
                    y: ((lparam.0 >> 16) as u16 as i16) as i32 * 96 / scale_dpi,
                };
                if p.y < 48 && p.x >= 348 {
                    hide_window(hwnd, &shared);
                } else if p.y < 48 && (250..300).contains(&p.x) {
                    shared.total.set(false);
                    (shared.events)(StatusEvent::PollUsageCosts { total: false });
                    unsafe {
                        let _ =
                            windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
                    }
                } else if p.y < 48 && (300..348).contains(&p.x) {
                    shared.total.set(true);
                    (shared.events)(StatusEvent::PollUsageCosts {
                        total: shared.total.get(),
                    });
                    unsafe {
                        let _ =
                            windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
                    }
                } else if (330..360).contains(&p.y) {
                    show_language_menu(hwnd, &shared);
                } else if p.y > 400 && p.x < 190 {
                    let text = shared.view.borrow().details.clone();
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            Some(hwnd),
                            &windows::core::HSTRING::from(text),
                            &windows::core::HSTRING::from("青简·详细记录"),
                            windows::Win32::UI::WindowsAndMessaging::MB_OK
                                | windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION,
                        );
                    }
                } else if p.y > 400 {
                    super::open_settings();
                }
                LRESULT(0)
            }
            WM_RBUTTONDOWN => {
                hide_window(hwnd, &shared);
                LRESULT(0)
            }
            WM_KEYDOWN if wparam.0 as u32 == VK_ESCAPE.0 as u32 => {
                hide_window(hwnd, &shared);
                LRESULT(0)
            }
            WM_TIMER if wparam.0 == TIMER_ID => {
                poll_timer(hwnd, &shared);
                LRESULT(0)
            }
            WM_COMMAND if (1..=5).contains(&(wparam.0 as u32)) => {
                let language = match wparam.0 as u32 {
                    1 => qingjian_core::Language::English,
                    2 => qingjian_core::Language::Japanese,
                    3 => qingjian_core::Language::Spanish,
                    4 => qingjian_core::Language::Vietnamese,
                    _ => qingjian_core::Language::German,
                };
                (shared.events)(StatusEvent::SelectLearningLanguage(language));
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
        }
    })
}

fn poll_timer(hwnd: HWND, shared: &Shared) {
    let mut ticks = shared.timer_ticks.get().wrapping_add(1);
    if ticks >= 10 {
        ticks = 0;
        (shared.events)(StatusEvent::PollUsageCosts {
            total: shared.total.get(),
        });
    }
    shared.timer_ticks.set(ticks);
    if shared.menu_open.get() {
        return;
    }
    let left = key_down(VK_LBUTTON.0 as i32);
    let right = key_down(VK_RBUTTON.0 as i32);
    let mut cursor = POINT::default();
    let outside = unsafe { GetCursorPos(&mut cursor).is_ok() }
        && unsafe { windows::Win32::UI::WindowsAndMessaging::WindowFromPoint(cursor) } != hwnd;
    let just_pressed =
        (left && !shared.left_was_down.get()) || (right && !shared.right_was_down.get());
    shared.left_was_down.set(left);
    shared.right_was_down.set(right);
    if (outside && just_pressed) || key_down(VK_ESCAPE.0 as i32) {
        hide_window(hwnd, shared);
    }
}

fn show_language_menu(hwnd: HWND, shared: &Shared) {
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_err() {
        return;
    }
    let menu = menu.unwrap();
    let options = [
        (1, "英语", qingjian_core::Language::English),
        (2, "日语", qingjian_core::Language::Japanese),
        (3, "西班牙语", qingjian_core::Language::Spanish),
        (4, "越南语", qingjian_core::Language::Vietnamese),
        (5, "德语", qingjian_core::Language::German),
    ];
    for (id, title, _) in options {
        let text = windows::core::HSTRING::from(title);
        unsafe {
            let _ = AppendMenuW(menu, MF_STRING, id as usize, PCWSTR(text.as_ptr()));
        }
    }
    let mut point = POINT::default();
    shared.menu_open.set(true);
    unsafe {
        let _ = GetCursorPos(&mut point);
        let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, None, hwnd, None);
        let _ = DestroyMenu(menu);
    }
    shared.menu_open.set(false);
}

fn paint(hwnd: HWND, shared: &Shared) {
    let mut paint = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
    if hdc.is_invalid() {
        let _ = unsafe { EndPaint(hwnd, &paint) };
        return;
    }
    paint_content(hdc, shared);
    let _ = unsafe { EndPaint(hwnd, &paint) };
}

fn paint_content(hdc: HDC, shared: &Shared) {
    let scale_dpi = shared.scale_dpi.get().max(1);
    let scale = |value: i32| value * scale_dpi as i32 / 96;
    let rect = RECT {
        left: 0,
        top: 0,
        right: scale(LOGICAL_WIDTH),
        bottom: scale(LOGICAL_HEIGHT),
    };
    let background = unsafe { CreateSolidBrush(COLORREF(0x00FFFFFF)) };
    unsafe {
        let _ = FillRect(hdc, &rect, background);
        let _ = DeleteObject(background.into());
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, TEXT);
    }
    let data = shared.view.borrow();
    draw_text(
        scale_dpi,
        hdc,
        "青简·用量与余额",
        (scale(20), scale(14), scale(220)),
        true,
        TEXT,
    );
    draw_text(
        scale_dpi,
        hdc,
        "今日",
        (scale(256), scale(18), scale(42)),
        false,
        if shared.total.get() { MUTED } else { BLUE },
    );
    draw_text(
        scale_dpi,
        hdc,
        "累计",
        (scale(302), scale(18), scale(42)),
        false,
        if shared.total.get() { BLUE } else { MUTED },
    );
    draw_text(
        scale_dpi,
        hdc,
        "×",
        (scale(354), scale(16), scale(24)),
        true,
        MUTED,
    );
    draw_text(
        scale_dpi,
        hdc,
        "本输入法已用 token",
        (scale(20), scale(60), scale(190)),
        false,
        MUTED,
    );
    draw_text(
        scale_dpi,
        hdc,
        &data.tokens,
        (scale(224), scale(60), scale(146)),
        true,
        TEXT,
    );
    let fields = [
        (data.token_detail.as_str(), 20, 94, 370, 141),
        (data.amount.as_str(), 20, 177, 370, 220),
        (data.balance.as_str(), 20, 254, 370, 281),
        (data.requests.as_str(), 20, 292, 370, 330),
        (data.updated.as_str(), 20, 372, 370, 404),
    ];
    for (value, left, top, right, bottom) in fields {
        let mut value_rect = RECT {
            left: scale(left),
            top: scale(top),
            right: scale(right),
            bottom: scale(bottom),
        };
        draw_wrapped(scale_dpi, hdc, value, &mut value_rect, TEXT);
    }
    draw_text(
        scale_dpi,
        hdc,
        "本输入法估算花费",
        (scale(20), scale(147), scale(340)),
        false,
        MUTED,
    );
    draw_text(
        scale_dpi,
        hdc,
        "DeepSeek 账户剩余",
        (scale(20), scale(223), scale(340)),
        false,
        MUTED,
    );
    draw_text(
        scale_dpi,
        hdc,
        &format!("目标语言：{}  ▾", data.target_language),
        (scale(20), scale(334), scale(340)),
        false,
        BLUE,
    );
    draw_text(
        scale_dpi,
        hdc,
        "详细记录",
        (scale(MARGIN), scale(410), scale(105)),
        false,
        BLUE,
    );
    draw_text(
        scale_dpi,
        hdc,
        "设置",
        (scale(315), scale(410), scale(50)),
        false,
        BLUE,
    );
}

fn draw_text(
    scale_dpi: u32,
    hdc: HDC,
    text: &str,
    bounds: (i32, i32, i32),
    bold: bool,
    color: COLORREF,
) {
    if text.is_empty() {
        return;
    }
    let (x, y, width) = bounds;
    let height = (15 * scale_dpi as i32 / 96).max(1);
    let font = unsafe {
        CreateFontW(
            -height,
            0,
            0,
            0,
            if bold { FW_BOLD.0 as i32 } else { 400 },
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_TT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Microsoft YaHei UI"),
        )
    };
    if font.is_invalid() {
        return;
    }
    let old = unsafe { SelectObject(hdc, font.into()) };
    unsafe {
        let _ = SetTextColor(hdc, color);
    }
    let mut rect = RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height * 2,
    };
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
    unsafe {
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }
}

fn draw_wrapped(scale_dpi: u32, hdc: HDC, text: &str, rect: &mut RECT, color: COLORREF) {
    if text.is_empty() {
        return;
    }
    let height = (14 * scale_dpi as i32 / 96).max(1);
    let font = unsafe {
        CreateFontW(
            -height,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_TT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (VARIABLE_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Microsoft YaHei UI"),
        )
    };
    if font.is_invalid() {
        return;
    }
    let old = unsafe { SelectObject(hdc, font.into()) };
    unsafe {
        let _ = SetTextColor(hdc, color);
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let _ = DrawTextW(hdc, &mut wide, rect, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
    }
    unsafe {
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_empty_snapshot_paints_before_statistics_arrive() {
        let window = CostWindow::new(Rc::new(|_| {})).unwrap();
        window.show(CostPanelView::default(), (0, 0));
        assert!(unsafe { windows::Win32::Graphics::Gdi::UpdateWindow(window.hwnd) }.as_bool());
    }
}
