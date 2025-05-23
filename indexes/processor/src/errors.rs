use thiserror::Error;
use vecno_notify::events::EventType;
use vecno_utxoindex::errors::UtxoIndexError;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("{0}")]
    UtxoIndexError(#[from] UtxoIndexError),

    #[error("event type {0:?} is not supported")]
    NotSupported(EventType),
}
pub type IndexResult<T> = std::result::Result<T, IndexError>;
