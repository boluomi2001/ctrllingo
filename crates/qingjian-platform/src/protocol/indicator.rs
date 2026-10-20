use serde::{Deserialize, Serialize};

/// 任务栏「中 / 英」图标右键菜单里要交给 Server 办的项（中 / 英切换在 DLL 侧自己做）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorCommand {
    /// 翻转当前模式的全角标点，与悬浮条上点「，。」一样。
    TogglePunctuation,

    /// 显示 / 隐藏悬浮状态条（`[status_bar] enabled`）。
    ToggleStatusBar,

    /// 打开设置程序。DLL 可能在 UWP 沙箱里起不了进程，交给 Server 起。
    OpenSettings,

    /// 打开旧版设置程序（`qingjian-settings-legacy.exe`，留一个版本作回退），同样交给 Server。
    OpenLegacySettings,

    /// 查到新版本时菜单里的「有新版本」：打开下载页，同样交给 Server。
    OpenDownload,

    /// 打开符号面板。
    OpenSymbolPanel,

    /// 打开用量与费用详情。
    OpenUsageCosts,

    OpenUsagePanel {
        x: i32,
        y: i32,
    },

    /// 请求 Server 在后台刷新账户余额。
    RefreshBalance,

    /// 打开 DeepSeek 用量页面。
    OpenBilling,
}

/// 右键菜单打勾用的开关状态。DLL 不读配置文件（UWP 沙箱里读不到），由 Server 随
/// [`super::ServerMessage::ModeSync`] 每一拍带下来。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct IndicatorState {
    /// 中文模式下标点转全角（`[general] full_width_punctuation`）。
    pub full_width_punctuation: bool,

    /// 英文模式下标点转全角（`[general] english_full_width_punctuation`）。
    pub english_full_width_punctuation: bool,

    /// 悬浮状态条开着（`[status_bar] enabled`）。
    pub status_bar: bool,

    /// 检查更新查到了新版本，菜单里露出「有新版本」。
    pub update_available: bool,

    /// 符号面板开着。从悬浮条打开时 DLL 靠这一拍得知，没在组句也把键交给 Server。
    pub symbol_panel: bool,

    /// 本输入法估算的用量与费用摘要；空字符串表示暂不可用。
    pub cost_summary: String,

    /// DeepSeek 账户余额摘要；空字符串表示暂不可用。
    pub balance_summary: String,
}
