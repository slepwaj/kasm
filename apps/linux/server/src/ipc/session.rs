//! 一条连接内的会话映射与独立协商状态。
use kasm_platform::protocol::SessionId;
use kasm_platform::protocol::linux::DisplayIdentity;

pub(super) struct Session {
    pub(super) global: SessionId,

    pub(super) identity: Option<DisplayIdentity>,

    pub(super) capabilities: bool,
}
