/*
 * 版本：0.0.1
 * 一个用rust实现区块链的简单例子，只是说明区块链的原理
 * 1. 仅有单个节点
 * 2. 过程式编程
 * 3. 零难度挖矿
 * 4. 每个块仅有一条交易记录
 */
use chrono::Utc;
use tracing::info;
use anyhow::Result;
use crypto::{sha3::Sha3, digest::Digest};
use serde::{Serialize, Deserialize};
use thiserror::Error;


/*
 * 区块链核心数据结构定义
 */

/* 区块头定义 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    timestamp: i64,     // 当前块的时间戳
    prev_hash: String,  // 前一个块的hash
    nonce: usize,       // 挖矿用随机数
}

/* 区块定义：包括区块头，区块体（data），当前区块hash */
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    header: BlockHeader,
    data: String,
    hash: String,
}

/* 区块链定义：包括区块数组，当前区块高度 */
pub struct Blockchain {
    blocks: Vec<Block>,
    height: usize,
}

/* 错误信息 */
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Serialize or Deserialize error")]
    SerializeError(#[from] Box<bincode::ErrorKind>),
}

/*
 * 区块链核心函数定义
 */

/* 区块体数据序列化 */
pub fn serialize<T>(data: &T) -> Result<Vec<u8>, BlockchainError>
where
    T: Serialize + ?Sized
{
    Ok(bincode::serialize(data)?)
}

/* 根据字节流计算hash值，用于计算一个区块的hash */
pub fn hash_to_str(data: &[u8]) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input(data);
    hasher.result_str()
}


/* 生成新的区块 */
pub fn new_block(data: &str, prev_hash: &str) -> Block {
    let mut block = Block {
        header: BlockHeader {
            timestamp: Utc::now().timestamp(),
            prev_hash: prev_hash.into(),
            nonce: 0,
        },
        data: data.into(),
        hash: String::new(),
    };

    if let Ok(serialized) = serialize(&block.header) {
        block.hash = hash_to_str(&serialized)
    }

    block
}

/* 生成创世块，prev_hash为空 */
pub fn create_genesis() -> Block {
    new_block("Genesis Block", "")
}

/* 生成新的区块链 */
pub fn create_blockchain() -> Blockchain {
    let bc = Blockchain {
        blocks: vec![create_genesis()],
        height: 0,
    };

    bc
}

/* 挖矿示例：生成一个新的区块并添加在当前区块链尾部 */
pub fn mining(bc: &mut Blockchain, data: &str) {
    let prev_block = bc.blocks.last().unwrap();
    let block = new_block(data, prev_block.hash.as_str());
    bc.blocks.push(block);
    bc.height += 1;
}

/* 打印区块链详细信息 */
pub fn blocks_info(bc: &Blockchain) {
    for block in bc.blocks.iter() {
        info!("{:#?}", block);
    }
}
      
