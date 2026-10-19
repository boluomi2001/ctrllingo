//! 费用摘要留在 Server，DLL 只接收文字，不持有密钥或发起网络请求。

mod monitor;
mod snapshot;
mod state;
mod view;

pub(super) use monitor::CostMonitor;

pub(super) fn beijing_now() -> jiff::Zoned {
    jiff::Timestamp::now().to_zoned(jiff::tz::Offset::from_hours(8).unwrap().to_time_zone())
}
