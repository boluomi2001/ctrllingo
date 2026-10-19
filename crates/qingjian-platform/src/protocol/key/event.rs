use serde::{Deserialize, Serialize};

use super::modifiers::KeyModifiers;

/// DLL 从 TSF `OnKeyDown` / `OnTestKeyDown` 抓到的一次按键，发给 Server 判定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    /// Windows 虚拟键码（`VK_*`）。翻页、方向键、退格、回车等靠它区分。
    pub virtual_key: u32,

    /// 这次按键产生的字符（`ToUnicode` 的结果）；功能键没有字符时为 `None`。
    pub character: Option<char>,

    /// 按下时的修饰键状态。
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Ctrl 单击完成的内部事件；真实 Windows 虚拟键码不超过 0xff。
    pub const SWAP_TRANSLATION: u32 = 0x100;

    pub fn new(virtual_key: u32, character: Option<char>, modifiers: KeyModifiers) -> Self {
        Self {
            virtual_key,
            character,
            modifiers,
        }
    }
}
