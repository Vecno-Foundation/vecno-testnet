use std::{str::FromStr, sync::Arc, time::Duration};

use crate::common::{client_notify::ChannelNotify, daemon::Daemon};
use futures_util::future::try_join_all;
use tokio::task::JoinHandle;
use vecno_addresses::{Address, Prefix, Version};
use vecno_consensus::params::SIMNET_GENESIS;
use vecno_consensus_core::{constants::MAX_SOMPI, subnets::SubnetworkId, tx::Transaction};
use vecno_core::info;
use vecno_grpc_core::ops::VecnodPayloadOps;
use vecno_hashes::Hash;
use vecno_notify::{
    connection::{ChannelConnection, ChannelType},
    scope::{
        BlockAddedScope, FinalityConflictScope, NewBlockTemplateScope, PruningPointUtxoSetOverrideScope, Scope,
        SinkBlueScoreChangedScope, UtxosChangedScope, VirtualChainChangedScope, VirtualDaaScoreChangedScope,
    },
};
use vecno_rpc_core::{api::rpc::RpcApi, model::*, Notification};
use vecno_utils::{fd_budget, networking::ContextualNetAddress};
use vecnod_lib::args::Args;

#[macro_export]
macro_rules! tst {
    ($op:ident, $test_body:block) => {
        tokio::spawn(async move {
            info!("Testing  {:?}", $op);
            $test_body
        })
    };

    ($op:ident, $reason:literal) => {
        tokio::spawn(async move {
            info!("Skipping {:?} --- {}", $op, $reason);
        })
    };
}

/// `cargo test --release --package vecno-testing-integration --lib -- rpc_tests::sanity_test`
#[tokio::test]
async fn sanity_test() {
    vecno_core::log::try_init_logger("info");
    // As we log the panic, we want to set it up after the logger
    vecno_core::panic::configure_panic();

    let args = Args {
        simnet: true,
        disable_upnp: true, // UPnP registration might take some time and is not needed for this test
        enable_unsynced_mining: true,
        block_template_cache_lifetime: Some(0),
        utxoindex: true,
        unsafe_rpc: true,
        ..Default::default()
    };

    let fd_total_budget = fd_budget::limit();
    let mut daemon = Daemon::new_random_with_args(args, fd_total_budget);
    let client = daemon.start().await;
    let (sender, _) = async_channel::unbounded();
    let connection = ChannelConnection::new("test", sender, ChannelType::Closable);
    let listener_id = client.register_new_listener(connection);
    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    // The intent of this for/match design (emphasizing the absence of an arm with fallback pattern in the match)
    // is to force any implementor of a new RpcApi method to add a matching arm here and to strongly incentivize
    // the adding of an actual sanity test of said new method.
    for op in VecnodPayloadOps::list() {
        let network_id = daemon.network;
        let task: JoinHandle<()> = match op {
            VecnodPayloadOps::SubmitBlock => {
                let rpc_client = client.clone();
                tst!(op, {
                    // Register to basic virtual events in order to keep track of block submission
                    let (sender, event_receiver) = async_channel::unbounded();
                    rpc_client.start(Some(Arc::new(ChannelNotify::new(sender)))).await;
                    rpc_client
                        .start_notify(Default::default(), Scope::VirtualDaaScoreChanged(VirtualDaaScoreChangedScope {}))
                        .await
                        .unwrap();

                    // Before submitting a first block, the sink is the genesis,
                    let response = rpc_client.get_sink_call(GetSinkRequest {}).await.unwrap();
                    assert_eq!(response.sink, SIMNET_GENESIS.hash);
                    let response = rpc_client.get_sink_blue_score_call(GetSinkBlueScoreRequest {}).await.unwrap();
                    assert_eq!(response.blue_score, 0);

                    // the block count is 0
                    let response = rpc_client.get_block_count_call(GetBlockCountRequest {}).await.unwrap();
                    assert_eq!(response.block_count, 0);

                    // and the virtual chain is the genesis only
                    let response = rpc_client
                        .get_virtual_chain_from_block_call(GetVirtualChainFromBlockRequest {
                            start_hash: SIMNET_GENESIS.hash,
                            include_accepted_transaction_ids: false,
                        })
                        .await
                        .unwrap();
                    assert!(response.added_chain_block_hashes.is_empty());
                    assert!(response.removed_chain_block_hashes.is_empty());

                    // Get a block template
                    let GetBlockTemplateResponse { block, is_synced } = rpc_client
                        .get_block_template_call(GetBlockTemplateRequest {
                            pay_address: Address::new(Prefix::Simnet, Version::PubKey, &[0u8; 32]),
                            extra_data: Vec::new(),
                        })
                        .await
                        .unwrap();
                    assert!(!is_synced);

                    // Submit the template (no mining, in simnet PoW is skipped)
                    let response = rpc_client.submit_block(block.clone(), false).await.unwrap();
                    assert_eq!(response.report, SubmitBlockReport::Success);

                    // Wait for virtual event indicating the block was processed and entered past(virtual)
                    while let Ok(notification) = match tokio::time::timeout(Duration::from_secs(1), event_receiver.recv()).await {
                        Ok(res) => res,
                        Err(elapsed) => panic!("expected virtual event before {}", elapsed),
                    } {
                        match notification {
                            Notification::VirtualDaaScoreChanged(msg) if msg.virtual_daa_score == 1 => {
                                break;
                            }
                            Notification::VirtualDaaScoreChanged(msg) if msg.virtual_daa_score > 1 => {
                                panic!("DAA score too high for number of submitted blocks")
                            }
                            Notification::VirtualDaaScoreChanged(_) => {}
                            _ => {}
                        }
                    }

                    // After submitting a first block, the sink is the submitted block,
                    let response = rpc_client.get_sink_call(GetSinkRequest {}).await.unwrap();
                    assert_eq!(response.sink, block.header.hash);

                    // the block count is 1
                    let response = rpc_client.get_block_count_call(GetBlockCountRequest {}).await.unwrap();
                    assert_eq!(response.block_count, 1);

                    // and the virtual chain from genesis contains the added block
                    let response = rpc_client
                        .get_virtual_chain_from_block_call(GetVirtualChainFromBlockRequest {
                            start_hash: SIMNET_GENESIS.hash,
                            include_accepted_transaction_ids: false,
                        })
                        .await
                        .unwrap();
                    assert!(response.added_chain_block_hashes.contains(&block.header.hash));
                    assert!(response.removed_chain_block_hashes.is_empty());
                })
            }

            VecnodPayloadOps::GetBlockTemplate => {
                tst!(op, "see SubmitBlock")
            }

            VecnodPayloadOps::GetCurrentNetwork => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_current_network_call(GetCurrentNetworkRequest {}).await.unwrap();
                    assert_eq!(response.network, network_id.network_type);
                })
            }

            VecnodPayloadOps::GetBlock => {
                let rpc_client = client.clone();
                tst!(op, {
                    let result = rpc_client.get_block_call(GetBlockRequest { hash: 0.into(), include_transactions: false }).await;
                    assert!(result.is_err());

                    let response = rpc_client
                        .get_block_call(GetBlockRequest { hash: SIMNET_GENESIS.hash, include_transactions: false })
                        .await
                        .unwrap();
                    assert_eq!(response.block.header.hash, SIMNET_GENESIS.hash);
                })
            }

            VecnodPayloadOps::GetBlocks => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client
                        .get_blocks_call(GetBlocksRequest { include_blocks: true, include_transactions: false, low_hash: None })
                        .await
                        .unwrap();
                    assert_eq!(response.blocks.len(), 1, "genesis block should be returned");
                    assert_eq!(response.blocks[0].header.hash, SIMNET_GENESIS.hash);
                    assert_eq!(response.block_hashes[0], SIMNET_GENESIS.hash);
                })
            }

            VecnodPayloadOps::GetInfo => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_info_call(GetInfoRequest {}).await.unwrap();
                    assert_eq!(response.server_version, vecno_core::vecnod_env::version().to_string());
                    assert_eq!(response.mempool_size, 0);
                    assert!(response.is_utxo_indexed);
                    assert!(response.has_message_id);
                    assert!(response.has_notify_command);
                })
            }

            VecnodPayloadOps::Shutdown => {
                // This test is purposely left blank since shutdown can only be tested after all other
                // tests completed
                tst!(op, "must be run in the end")
            }

            VecnodPayloadOps::GetPeerAddresses => {
                tst!(op, "see AddPeer, Ban")
            }

            VecnodPayloadOps::GetSink => {
                tst!(op, "see SubmitBlock")
            }

            VecnodPayloadOps::GetMempoolEntry => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response_result = rpc_client
                        .get_mempool_entry_call(GetMempoolEntryRequest {
                            transaction_id: 0.into(),
                            include_orphan_pool: true,
                            filter_transaction_pool: false,
                        })
                        .await;
                    // Test Get Mempool Entry:
                    // TODO: Fix by adding actual mempool entries this can get because otherwise it errors out
                    assert!(response_result.is_err());
                })
            }

            VecnodPayloadOps::GetMempoolEntries => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client
                        .get_mempool_entries_call(GetMempoolEntriesRequest {
                            include_orphan_pool: true,
                            filter_transaction_pool: false,
                        })
                        .await
                        .unwrap();
                    assert!(response.mempool_entries.is_empty());
                })
            }

            VecnodPayloadOps::GetConnectedPeerInfo => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_connected_peer_info_call(GetConnectedPeerInfoRequest {}).await.unwrap();
                    assert!(response.peer_info.is_empty());
                })
            }

            VecnodPayloadOps::AddPeer => {
                let rpc_client = client.clone();
                tst!(op, {
                    let peer_address = ContextualNetAddress::from_str("1.2.3.4").unwrap();
                    let _ = rpc_client.add_peer_call(AddPeerRequest { peer_address, is_permanent: true }).await.unwrap();

                    // Add peer only adds the IP to a connection request. It will only be added to known_addresses if it
                    // actually can be connected to. So in this test we can't expect it to be added unless we set up an
                    // actual peer.
                    let response = rpc_client.get_peer_addresses_call(GetPeerAddressesRequest {}).await.unwrap();
                    assert!(response.known_addresses.is_empty());
                })
            }

            VecnodPayloadOps::Ban => {
                let rpc_client = client.clone();
                tst!(op, {
                    let peer_address = ContextualNetAddress::from_str("5.6.7.8").unwrap();
                    let ip = peer_address.normalize(1).ip;

                    let _ = rpc_client.add_peer_call(AddPeerRequest { peer_address, is_permanent: false }).await.unwrap();
                    let _ = rpc_client.ban_call(BanRequest { ip }).await.unwrap();

                    let response = rpc_client.get_peer_addresses_call(GetPeerAddressesRequest {}).await.unwrap();
                    assert!(response.banned_addresses.contains(&ip));

                    let _ = rpc_client.unban_call(UnbanRequest { ip }).await.unwrap();
                    let response = rpc_client.get_peer_addresses_call(GetPeerAddressesRequest {}).await.unwrap();
                    assert!(!response.banned_addresses.contains(&ip));
                })
            }

            VecnodPayloadOps::Unban => {
                tst!(op, "see Ban")
            }

            VecnodPayloadOps::SubmitTransaction => {
                let rpc_client = client.clone();
                tst!(op, {
                    // Build an erroneous transaction...
                    let transaction = Transaction::new(0, vec![], vec![], 0, SubnetworkId::default(), 0, vec![]);
                    let result = rpc_client.submit_transaction((&transaction).into(), false).await;
                    // ...that gets rejected by the consensus
                    assert!(result.is_err());
                })
            }

            VecnodPayloadOps::GetSubnetwork => {
                let rpc_client = client.clone();
                tst!(op, {
                    let result =
                        rpc_client.get_subnetwork_call(GetSubnetworkRequest { subnetwork_id: SubnetworkId::from_byte(0) }).await;

                    // Err because it's currently unimplemented
                    assert!(result.is_err());
                })
            }

            VecnodPayloadOps::GetVirtualChainFromBlock => {
                tst!(op, "see SubmitBlock")
            }

            VecnodPayloadOps::GetBlockCount => {
                tst!(op, "see SubmitBlock")
            }

            VecnodPayloadOps::GetBlockDagInfo => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_block_dag_info_call(GetBlockDagInfoRequest {}).await.unwrap();
                    assert_eq!(response.network, network_id);
                })
            }

            VecnodPayloadOps::ResolveFinalityConflict => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response_result = rpc_client
                        .resolve_finality_conflict_call(ResolveFinalityConflictRequest {
                            finality_block_hash: Hash::from_bytes([0; 32]),
                        })
                        .await;

                    // Err because it's currently unimplemented
                    assert!(response_result.is_err());
                })
            }

            VecnodPayloadOps::GetHeaders => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response_result = rpc_client
                        .get_headers_call(GetHeadersRequest { start_hash: SIMNET_GENESIS.hash, limit: 1, is_ascending: true })
                        .await;

                    // Err because it's currently unimplemented
                    assert!(response_result.is_err());
                })
            }

            VecnodPayloadOps::GetUtxosByAddresses => {
                let rpc_client = client.clone();
                tst!(op, {
                    let addresses = vec![Address::new(Prefix::Simnet, Version::PubKey, &[0u8; 32])];
                    let response = rpc_client.get_utxos_by_addresses_call(GetUtxosByAddressesRequest { addresses }).await.unwrap();
                    assert!(response.entries.is_empty());
                })
            }

            VecnodPayloadOps::GetBalanceByAddress => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client
                        .get_balance_by_address_call(GetBalanceByAddressRequest {
                            address: Address::new(Prefix::Simnet, Version::PubKey, &[0u8; 32]),
                        })
                        .await
                        .unwrap();
                    assert_eq!(response.balance, 0);
                })
            }

            VecnodPayloadOps::GetBalancesByAddresses => {
                let rpc_client = client.clone();
                tst!(op, {
                    let addresses = vec![Address::new(Prefix::Simnet, Version::PubKey, &[1u8; 32])];
                    let response = rpc_client
                        .get_balances_by_addresses_call(GetBalancesByAddressesRequest::new(addresses.clone()))
                        .await
                        .unwrap();
                    assert_eq!(response.entries.len(), 1);
                    assert_eq!(response.entries[0].address, addresses[0]);
                    assert_eq!(response.entries[0].balance, Some(0));

                    let response =
                        rpc_client.get_balances_by_addresses_call(GetBalancesByAddressesRequest::new(vec![])).await.unwrap();
                    assert!(response.entries.is_empty());
                })
            }

            VecnodPayloadOps::GetSinkBlueScore => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_sink_blue_score_call(GetSinkBlueScoreRequest {}).await.unwrap();
                    // A concurrent test may have added a single block so the blue score can be either 0 or 1
                    assert!(response.blue_score < 2);
                })
            }

            VecnodPayloadOps::EstimateNetworkHashesPerSecond => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response_result = rpc_client
                        .estimate_network_hashes_per_second_call(EstimateNetworkHashesPerSecondRequest {
                            window_size: 1000,
                            start_hash: None,
                        })
                        .await;
                    // The current DAA window is almost empty so an error is expected
                    assert!(response_result.is_err());
                })
            }

            VecnodPayloadOps::GetMempoolEntriesByAddresses => {
                let rpc_client = client.clone();
                tst!(op, {
                    let addresses = vec![Address::new(Prefix::Simnet, Version::PubKey, &[0u8; 32])];
                    let response = rpc_client
                        .get_mempool_entries_by_addresses_call(GetMempoolEntriesByAddressesRequest::new(
                            addresses.clone(),
                            true,
                            false,
                        ))
                        .await
                        .unwrap();
                    assert_eq!(response.entries.len(), 1);
                    assert_eq!(response.entries[0].address, addresses[0]);
                    assert!(response.entries[0].receiving.is_empty());
                    assert!(response.entries[0].sending.is_empty());
                })
            }

            VecnodPayloadOps::GetCoinSupply => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_coin_supply_call(GetCoinSupplyRequest {}).await.unwrap();
                    assert_eq!(response.circulating_sompi, 0);
                    assert_eq!(response.max_sompi, MAX_SOMPI);
                })
            }

            VecnodPayloadOps::Ping => {
                let rpc_client = client.clone();
                tst!(op, {
                    let _ = rpc_client.ping_call(PingRequest {}).await.unwrap();
                })
            }

            VecnodPayloadOps::GetMetrics => {
                let rpc_client = client.clone();
                tst!(op, {
                    let get_metrics_call_response = rpc_client
                        .get_metrics_call(GetMetricsRequest {
                            consensus_metrics: true,
                            connection_metrics: true,
                            bandwidth_metrics: true,
                            process_metrics: true,
                        })
                        .await
                        .unwrap();
                    assert!(get_metrics_call_response.process_metrics.is_some());
                    assert!(get_metrics_call_response.consensus_metrics.is_some());

                    let get_metrics_call_response = rpc_client
                        .get_metrics_call(GetMetricsRequest {
                            consensus_metrics: false,
                            connection_metrics: true,
                            bandwidth_metrics: true,
                            process_metrics: true,
                        })
                        .await
                        .unwrap();
                    assert!(get_metrics_call_response.process_metrics.is_some());
                    assert!(get_metrics_call_response.consensus_metrics.is_none());

                    let get_metrics_call_response = rpc_client
                        .get_metrics_call(GetMetricsRequest {
                            consensus_metrics: true,
                            connection_metrics: true,
                            bandwidth_metrics: false,
                            process_metrics: false,
                        })
                        .await
                        .unwrap();
                    assert!(get_metrics_call_response.process_metrics.is_none());
                    assert!(get_metrics_call_response.consensus_metrics.is_some());

                    let get_metrics_call_response = rpc_client
                        .get_metrics_call(GetMetricsRequest {
                            consensus_metrics: false,
                            connection_metrics: true,
                            bandwidth_metrics: false,
                            process_metrics: false,
                        })
                        .await
                        .unwrap();
                    assert!(get_metrics_call_response.process_metrics.is_none());
                    assert!(get_metrics_call_response.consensus_metrics.is_none());
                })
            }

            VecnodPayloadOps::GetServerInfo => {
                let rpc_client = client.clone();
                tst!(op, {
                    let response = rpc_client.get_server_info_call(GetServerInfoRequest {}).await.unwrap();
                    assert!(response.has_utxo_index); // we set utxoindex above
                    assert_eq!(response.network_id, network_id);
                })
            }

            VecnodPayloadOps::GetSyncStatus => {
                let rpc_client = client.clone();
                tst!(op, {
                    let _ = rpc_client.get_sync_status_call(GetSyncStatusRequest {}).await.unwrap();
                })
            }

            VecnodPayloadOps::GetDaaScoreTimestampEstimate => {
                let rpc_client = client.clone();
                tst!(op, {
                    let results = rpc_client
                        .get_daa_score_timestamp_estimate_call(GetDaaScoreTimestampEstimateRequest {
                            daa_scores: vec![0, 500, 2000, u64::MAX],
                        })
                        .await
                        .unwrap();

                    for timestamp in results.timestamps.iter() {
                        info!("Timestamp estimate is {}", timestamp);
                    }

                    let results = rpc_client
                        .get_daa_score_timestamp_estimate_call(GetDaaScoreTimestampEstimateRequest { daa_scores: vec![] })
                        .await
                        .unwrap();

                    for timestamp in results.timestamps.iter() {
                        info!("Timestamp estimate is {}", timestamp);
                    }
                })
            }

            VecnodPayloadOps::NotifyBlockAdded => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, BlockAddedScope {}.into()).await.unwrap();
                })
            }

            VecnodPayloadOps::NotifyNewBlockTemplate => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, NewBlockTemplateScope {}.into()).await.unwrap();
                })
            }

            VecnodPayloadOps::NotifyFinalityConflict => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, FinalityConflictScope {}.into()).await.unwrap();
                })
            }
            VecnodPayloadOps::NotifyUtxosChanged => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, UtxosChangedScope::new(vec![]).into()).await.unwrap();
                })
            }
            VecnodPayloadOps::NotifySinkBlueScoreChanged => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, SinkBlueScoreChangedScope {}.into()).await.unwrap();
                })
            }
            VecnodPayloadOps::NotifyPruningPointUtxoSetOverride => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, PruningPointUtxoSetOverrideScope {}.into()).await.unwrap();
                })
            }
            VecnodPayloadOps::NotifyVirtualDaaScoreChanged => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.start_notify(id, VirtualDaaScoreChangedScope {}.into()).await.unwrap();
                })
            }
            VecnodPayloadOps::NotifyVirtualChainChanged => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client
                        .start_notify(id, VirtualChainChangedScope { include_accepted_transaction_ids: false }.into())
                        .await
                        .unwrap();
                })
            }
            VecnodPayloadOps::StopNotifyingUtxosChanged => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.stop_notify(id, UtxosChangedScope::new(vec![]).into()).await.unwrap();
                })
            }
            VecnodPayloadOps::StopNotifyingPruningPointUtxoSetOverride => {
                let rpc_client = client.clone();
                let id = listener_id;
                tst!(op, {
                    rpc_client.stop_notify(id, PruningPointUtxoSetOverrideScope {}.into()).await.unwrap();
                })
            }
        };
        tasks.push(task);
    }

    let _results = try_join_all(tasks).await;

    // Unregister the notification listener
    assert!(client.unregister_listener(listener_id).await.is_ok());

    // Shutdown should only be tested after everything
    let rpc_client = client.clone();
    let _ = rpc_client.shutdown_call(ShutdownRequest {}).await.unwrap();

    //
    // Fold-up
    //
    client.disconnect().await.unwrap();
    drop(client);
    daemon.shutdown();
}
