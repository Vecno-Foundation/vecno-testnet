#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    GrpcApi(#[from] vecno_rpc_core::error::RpcError),

    #[error(transparent)]
    GrpcClient(#[from] vecno_grpc_client::error::Error),

    #[error(transparent)]
    Wrpc(#[from] vecno_wrpc_server::error::Error),

    #[error(transparent)]
    WebSocket(#[from] workflow_rpc::server::WebSocketError),

    #[error(transparent)]
    WorkflowRpc(#[from] workflow_rpc::error::Error),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}
