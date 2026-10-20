//! 模式图标右键直接打开 Server 的费用卡片，不在应用进程中查余额。

use windows::Win32::Foundation::POINT;

use qingjian_platform::protocol::IndicatorCommand;

use super::TextService_Impl;
use crate::com::log::log;

impl TextService_Impl {
    pub(super) fn show_indicator_menu(&self, point: POINT) {
        match self.engine.borrow_mut().as_mut() {
            Some(client) => {
                if let Err(error) = client.indicator(IndicatorCommand::OpenUsagePanel {
                    x: point.x,
                    y: point.y,
                }) {
                    log(&format!("打开费用卡片失败: {error}"));
                }
            }
            None => log("费用卡片：没连上 Server"),
        }
    }
}
