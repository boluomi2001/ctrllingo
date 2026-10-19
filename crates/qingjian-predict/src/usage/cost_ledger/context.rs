//! 请求开始与结果记录之间传递的账本标识。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptContext {
    pub id: String,

    pub day: String,
}
