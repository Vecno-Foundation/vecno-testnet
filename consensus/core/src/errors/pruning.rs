use crate::BlockLevel;

use super::{block::RuleError, tx::TxRuleError};
use thiserror::Error;
use vecno_hashes::Hash;

#[derive(Error, Debug, Clone)]
pub enum PruningImportError {
    #[error("pruning proof validation failed")]
    ProofValidationError,

    #[error("pruning proof doesn't have {0} levels")]
    ProofNotEnoughLevels(usize),

    #[error("block {0} level is {1} when it's expected to be at least {2}")]
    PruningProofWrongBlockLevel(Hash, BlockLevel, BlockLevel),

    #[error("the proof header {0} is missing known parents at level {1}")]
    PruningProofHeaderWithNoKnownParents(Hash, BlockLevel),

    #[error("proof level {0} is missing the block at depth m in level {1}")]
    PruningProofMissingBlockAtDepthMFromNextLevel(BlockLevel, BlockLevel),

    #[error("the selected tip {0} at level {1} is not a parent of the pruning point")]
    PruningProofMissesBlocksBelowPruningPoint(Hash, BlockLevel),

    #[error("the pruning proof selected tip {0} at level {1} is not the pruning point")]
    PruningProofSelectedTipIsNotThePruningPoint(Hash, BlockLevel),

    #[error("the pruning proof selected tip {0} at level {1} is not a parent of the pruning point on the same level")]
    PruningProofSelectedTipNotParentOfPruningPoint(Hash, BlockLevel),

    #[error("the proof doesn't have sufficient blue work in order to replace the current DAG")]
    PruningProofInsufficientBlueWork,

    #[error("the pruning proof doesn't have any shared blocks with the known DAGs, but doesn't have enough headers from levels higher than the existing block levels.")]
    PruningProofNotEnoughHeaders,

    #[error("block {0} already appeared in the proof headers for level {1}")]
    PruningProofDuplicateHeaderAtLevel(Hash, BlockLevel),

    #[error("got header-only trusted block {0} which is not in pruning point past according to available reachability")]
    PruningPointPastMissingReachability(Hash),

    #[error("new pruning point has an invalid transaction {0}: {1}")]
    NewPruningPointTxError(Hash, TxRuleError),

    #[error("new pruning point has some invalid transactions")]
    NewPruningPointTxErrors,

    #[error("new pruning point transaction {0} is missing a UTXO entry")]
    NewPruningPointTxMissingUTXOEntry(Hash),

    #[error("the imported multiset hash was expected to be {0} and was actually {1}")]
    ImportedMultisetHashMismatch(Hash, Hash),

    #[error("pruning import data lead to validation rule error")]
    PruningImportRuleError(#[from] RuleError),

    #[error("process exit was initiated while validating pruning point proof")]
    PruningValidationInterrupted,
}

pub type PruningImportResult<T> = std::result::Result<T, PruningImportError>;
