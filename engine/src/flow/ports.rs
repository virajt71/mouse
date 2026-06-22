use crate::flow::network::FlowEvent;

#[async_trait::async_trait]
pub trait PeerTransport: Send + Sync {
    async fn send_event(&self, peer_name: &str, event: &FlowEvent) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait InputSimulator: Send + Sync {
    async fn inject_mouse_move(&self, dx: i32, dy: i32) -> anyhow::Result<()>;
    async fn inject_button(&self, code: u16, value: i32) -> anyhow::Result<()>;
    async fn inject_key(&self, code: u16, value: i32) -> anyhow::Result<()>;
    async fn inject_scroll(&self, horizontal: bool, delta: i32) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait WindowSystemQuery: Send + Sync {
    async fn query_cursor(&self) -> anyhow::Result<(i32, i32)>;
    async fn warp_cursor(&self, x: i32, y: i32) -> anyhow::Result<()>;
}
