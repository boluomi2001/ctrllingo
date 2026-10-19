//! 费用详情使用独立原生窗口线程，保持输入与候选窗口的消息循环畅通。

use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::UI::WindowsAndMessaging::{
    MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MessageBoxW,
};
use windows::core::HSTRING;

static OPEN: AtomicBool = AtomicBool::new(false);

pub(super) fn show(text: String) {
    if OPEN.swap(true, Ordering::AcqRel) {
        return;
    }
    let result = std::thread::Builder::new()
        .name("qingjian-cost-window".to_owned())
        .spawn(move || {
            unsafe {
                MessageBoxW(
                    None,
                    &HSTRING::from(text),
                    &HSTRING::from("青简双语测试 · 用量与费用"),
                    MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND,
                );
            }
            OPEN.store(false, Ordering::Release);
        });
    if result.is_err() {
        OPEN.store(false, Ordering::Release);
    }
}
