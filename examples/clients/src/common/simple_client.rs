use std::sync::{Arc, OnceLock};

use rmcp::{
    model::{ClientInfo, ServerInfo},
    service::ServerSink,
    ClientHandler,
};

#[derive(Debug, Clone, Default)]
pub struct SimpleClient {
    server_sink: Arc<OnceLock<ServerSink>>,
    server_info: Arc<OnceLock<ServerInfo>>,
}

impl ClientHandler for SimpleClient {
    fn get_peer(&self) -> Option<ServerSink> {
        self.server_sink.get().cloned()
    }

    fn set_peer(&mut self, peer: ServerSink) {
        self.server_sink.get_or_init(|| peer);
    }

    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }

    fn set_peer_info(&mut self, peer: ServerInfo) {
        self.server_info.get_or_init(|| peer);
    }

    fn get_peer_info(&self) -> Option<ServerInfo> {
        self.server_info.get().cloned()
    }
}
