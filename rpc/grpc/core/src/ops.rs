use crate::protowire::{vecnod_request::Payload as RequestPayload, vecnod_response::Payload as ResponsePayload, *};
use vecno_rpc_core::RpcError;
use workflow_core::enums::Describe;

macro_rules! payload_type_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$variant_meta:meta])* $variant_name:ident $(= $zero:literal)?,)*
    }) => {
        paste::paste! {
            $(#[$meta])*
            $vis enum $name {
                $($(#[$variant_meta])* $variant_name $(= $zero)?),*
            }

            impl $name {
                pub fn to_error_response(&self, error: RpcError) -> ResponsePayload {
                    match self {
                        $($name::$variant_name => [<$variant_name ResponseMessage>]::from(error).into()),*
                    }
                }
            }

            impl std::convert::From<&RequestPayload> for $name {
                fn from(value: &RequestPayload) -> Self {
                    match value {
                        $(RequestPayload::[<$variant_name Request>](_) => $name::$variant_name),*
                    }
                }
            }

            impl TryFrom<&ResponsePayload> for $name {
                type Error = ();

                fn try_from(value: &ResponsePayload) -> Result<Self, Self::Error> {
                    match value {
                        $(ResponsePayload::[<$variant_name Response>](_) => Ok($name::$variant_name)),*,
                        _ => Err(())
                    }
                }
            }

        }
    }
}

payload_type_enum! {
#[repr(u8)]
#[derive(Describe, Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum VecnodPayloadOps {
    SubmitBlock = 0,
    GetBlockTemplate,
    GetCurrentNetwork,
    GetBlock,
    GetBlocks,
    GetInfo,
    Shutdown,
    GetPeerAddresses,
    GetSink,
    GetMempoolEntry,
    GetMempoolEntries,
    GetConnectedPeerInfo,
    AddPeer,
    SubmitTransaction,
    GetSubnetwork,
    GetVirtualChainFromBlock,
    GetBlockCount,
    GetBlockDagInfo,
    ResolveFinalityConflict,
    GetHeaders,
    GetUtxosByAddresses,
    GetBalanceByAddress,
    GetBalancesByAddresses,
    GetSinkBlueScore,
    Ban,
    Unban,
    EstimateNetworkHashesPerSecond,
    GetMempoolEntriesByAddresses,
    GetCoinSupply,
    Ping,
    GetMetrics,
    GetServerInfo,
    GetSyncStatus,
    GetDaaScoreTimestampEstimate,

    // Subscription commands for starting/stopping notifications
    NotifyBlockAdded,
    NotifyNewBlockTemplate,
    NotifyFinalityConflict,
    NotifyUtxosChanged,
    NotifySinkBlueScoreChanged,
    NotifyPruningPointUtxoSetOverride,
    NotifyVirtualDaaScoreChanged,
    NotifyVirtualChainChanged,

    // Legacy stop subscription commands
    StopNotifyingUtxosChanged,
    StopNotifyingPruningPointUtxoSetOverride,

    // Please note:
    // Notification payloads existing in ResponsePayload are not considered valid ops.
    // The conversion from a notification ResponsePayload into VecnodPayloadOps fails.
}
}
