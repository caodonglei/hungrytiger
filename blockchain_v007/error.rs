use thiserror::Error;

/* 错误信息 */
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Serialize or Deserialize error")]
    SerializeError(#[from] Box<bincode::ErrorKind>),

    #[error("Failed to access rocks db")]
    RocksDbError(#[from] rocksdb::Error),
}

