use std::{sync::Arc, time::Duration};
use vecno_core::debug;
use vecno_p2p_lib::echo::EchoFlowInitializer;

#[tokio::main]
async fn main() {
    // [-] - init logger
    vecno_core::log::init_logger(None, "debug");
    // [0] - init p2p-adaptor
    let initializer = Arc::new(EchoFlowInitializer::new());
    let adaptor = vecno_p2p_lib::Adaptor::client_only(vecno_p2p_lib::Hub::new(), initializer, Default::default());
    // [1] - connect 128 peers + flows
    let ip_port = String::from("[::1]:50051");
    for i in 0..1 {
        debug!("P2P, p2p_client::main - starting peer:{}", i);
        let _peer_key = adaptor.connect_peer_with_retries(ip_port.clone(), 16, Duration::from_secs(1)).await;
    }
    // [2] - wait a few seconds and terminate
    tokio::time::sleep(Duration::from_secs(5)).await;
    debug!("P2P,p2p_client::main - TERMINATE");
    adaptor.terminate_all_peers().await;
    debug!("P2P,p2p_client::main - FINISH");
    tokio::time::sleep(Duration::from_secs(10)).await;
    debug!("P2P,p2p_client::main - EXIT");
}
