//! 单次请求的账本状态。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum AttemptState {
    Pending,
    Finished,
}
