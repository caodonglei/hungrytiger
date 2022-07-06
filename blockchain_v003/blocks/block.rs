use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::{ProofOfWork};

/* 区块头定义 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct BlockHeader {
    timestamp: i64,     // 当前块的时间戳
    prev_hash: String,  // 前一个块的hash
    bits: usize,        // 工作量证明的难度，hash的前bits位必须为0
    nonce: usize,       // 迭代挖矿的次数
}

/* 区块头功能封装 */
impl BlockHeader {
    // 构造函数
    fn new(prev_hash: &str, bits: usize) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            prev_hash: prev_hash.into(),
            bits,
            nonce: 0,
        }
    }
}

/* 区块定义：包括区块头，区块体（data），当前区块hash */
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Block {
    header: BlockHeader,
    data: String,
    hash: String,
}

/* 区块功能封装 */
impl Block {
    // 构造函数, 生成新的区块
    pub fn new(data: &str, pre_hash: &str, bits: usize) -> Self {
        let mut block = Block {
            header: BlockHeader::new(pre_hash, bits),
            data: data.into(),
            hash: String::new(),
        };

        // 工作量证明
        let pow = ProofOfWork::new(bits);
        pow.run(&mut block);

        block
    }

    // 静态方法：生成创世块，prev_hash为空
    pub fn create_genesis(bits: usize) -> Self {
        Self::new("Genesis Block", "", bits)
    }

    pub fn set_nonce(&mut self, nonce: usize) {
        self.header.nonce = nonce;
    }

    pub fn get_header(&self) -> BlockHeader {
        self.header.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn set_hash(&mut self, hash: String) {
        self.hash = hash;
    }

    pub fn get_prev_hash(&self) -> String {
        self.header.prev_hash.clone()
    }
}
