//! 青简独立托盘入口，点击后从鼠标位置打开用量卡片。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, IDI_APPLICATION, LoadIconW,
    RegisterWindowMessageW, WM_APP, WM_CONTEXTMENU, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW,
};
use windows::core::{PCWSTR, Result, w};

use super::window_class::WindowClass;
use crate::dispatch::StatusEvent;

const CLASS_NAME: PCWSTR = w!("QingjianTrayMessageWindow");
const CALLBACK_MESSAGE: u32 = WM_APP + 0x71;
const ICON_ID: u32 = 1;
static CLASS: WindowClass = WindowClass::new();

struct Shared {
    events: Rc<dyn Fn(StatusEvent)>,
    taskbar_created: u32,
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
}

thread_local! {
    static WINDOWS: RefCell<HashMap<isize, Rc<Shared>>> = RefCell::new(HashMap::new());
}

/// 系统托盘图标及其隐藏消息窗口。
pub(crate) struct TrayIcon {
    hwnd: HWND,
    data: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub(crate) fn new(events: Rc<dyn Fn(StatusEvent)>) -> Result<Self> {
        CLASS.ensure(|| WNDCLASSEXW {
            lpfnWndProc: Some(wndproc),
            hInstance: super::module_handle(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        })?;
        let taskbar_created = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        let icon = unsafe {
            LoadIconW(
                Some(super::module_handle()),
                PCWSTR(core::ptr::without_provenance::<u16>(1)),
            )
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))?
        };
        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                CLASS_NAME,
                w!("青简托盘消息窗口"),
                Default::default(),
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
        WINDOWS.with(|windows| {
            windows.borrow_mut().insert(
                hwnd.0 as isize,
                Rc::new(Shared {
                    events,
                    taskbar_created,
                    icon,
                }),
            );
        });
        let data = notify_data(hwnd, icon);
        let added = unsafe { Shell_NotifyIconW(NIM_ADD, &data).as_bool() };
        if !added {
            WINDOWS.with(|windows| {
                windows.borrow_mut().remove(&(hwnd.0 as isize));
            });
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return Err(windows::core::Error::from_thread());
        }
        Ok(Self { hwnd, data })
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.data);
        }
        WINDOWS.with(|windows| {
            windows.borrow_mut().remove(&(self.hwnd.0 as isize));
        });
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn notify_data(
    hwnd: HWND,
    icon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: core::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: ICON_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: CALLBACK_MESSAGE,
        hIcon: icon,
        ..Default::default()
    };
    let tip: Vec<u16> = "青简双语测试".encode_utf16().collect();
    for (target, value) in data.szTip.iter_mut().zip(tip) {
        *target = value;
    }
    data
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
        if message == shared.taskbar_created {
            let data = notify_data(hwnd, shared.icon);
            unsafe {
                let _ = Shell_NotifyIconW(NIM_ADD, &data);
            }
            return LRESULT(0);
        }
        if message == CALLBACK_MESSAGE
            && (lparam.0 as u32 == WM_RBUTTONUP
                || lparam.0 as u32 == WM_CONTEXTMENU
                || lparam.0 as u32 == WM_LBUTTONUP)
        {
            let mut point = POINT::default();
            if unsafe { GetCursorPos(&mut point) }.is_ok() {
                (shared.events)(StatusEvent::OpenUsageCosts(point.x, point.y));
            }
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    })
}
