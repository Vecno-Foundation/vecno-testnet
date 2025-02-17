use super::{
    handler_trait::Handler,
    interface::{DynVecnodMethod, Interface},
};
use crate::{
    connection::{Connection, IncomingRoute},
    connection_handler::ServerContext,
    error::GrpcServerResult,
};
use vecno_core::debug;
use vecno_grpc_core::{
    ops::VecnodPayloadOps,
    protowire::{VecnodRequest, VecnodResponse},
};

pub struct RequestHandler {
    rpc_op: VecnodPayloadOps,
    incoming_route: IncomingRoute,
    server_ctx: ServerContext,
    method: DynVecnodMethod,
    connection: Connection,
}

impl RequestHandler {
    pub fn new(
        rpc_op: VecnodPayloadOps,
        incoming_route: IncomingRoute,
        server_context: ServerContext,
        interface: &Interface,
        connection: Connection,
    ) -> Self {
        let method = interface.get_method(&rpc_op);
        Self { rpc_op, incoming_route, server_ctx: server_context, method, connection }
    }

    pub async fn handle_request(&self, request: VecnodRequest) -> GrpcServerResult<VecnodResponse> {
        let id = request.id;
        let mut response = self.method.call(self.server_ctx.clone(), self.connection.clone(), request).await?;
        response.id = id;
        Ok(response)
    }
}

#[async_trait::async_trait]
impl Handler for RequestHandler {
    async fn start(&mut self) {
        debug!("GRPC, Starting request handler {:?} for client {}", self.rpc_op, self.connection);
        while let Ok(request) = self.incoming_route.recv().await {
            let response = self.handle_request(request).await;
            match response {
                Ok(response) => {
                    if self.connection.enqueue(response).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    debug!("GRPC, Request handling error {} for client {}", e, self.connection);
                }
            }
        }
        debug!("GRPC, Exiting request handler {:?} for client {}", self.rpc_op, self.connection);
    }
}
