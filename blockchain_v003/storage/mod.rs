use crate::{Block, error::BlockchainError};

mod dbstore;

pub use dbstore::RocksDb;

// 常量定义
// 存储区块链最后一个block的hash，对应的key
pub const TIP_KEY: &str = "tip_hash";
// 存储区块高度，对应的key
pub const HEIGHT: &str = "height";
// 存储区块的key
pub const TABLE_OF_BLOCK: &str = "blocks";

/*
 * 数据库接口定义
 */
pub trait KVStorage: Send + Sync + 'static {
    fn get_tip(&self) -> Result<Option<String>, BlockchainError>;
    fn get_block(&self, key: &str) -> Result<Option<Block>, BlockchainError>;
    fn get_height(&self) -> Result<Option<usize>, BlockchainError>;
    fn update_blocks(&self, key: &str, block: &Block, height: usize);
}

pub struct KVStorageIterator<T> {
    data: T
}

impl<T> KVStorageIterator<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Iterator for KVStorageIterator<T>
where
    T: Iterator,
    T::Item: Into<Block>
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|v| v.into())
    }
}
