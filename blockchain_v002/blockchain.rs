/*
 * 版本：0.0.2
 * 一个用rust实现区块链的简单例子，只是说明区块链的原理
 * 1. 仅有单个节点
 * 2. 对象式编程
 * 3. PoW挖矿
 * 4. 每个块仅有一条交易记录
 */
use chrono::Utc;
use std::ops::Shl;
use bigint::U256;
use tracing::info;
use anyhow::Result;
use crypto::{sha3::Sha3, digest::Digest};
use serde::{Serialize, Deserialize};
use thiserror::Error;

// 常量，工作量证明的难度
const CURR_BITS: usize = 2;
// 挖矿最大迭代次数
const MAX_NONCE: usize = usize::MAX;

/*
 * 区块链核心数据结构定义
 */

/* 区块头定义 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
}

/* 区块链定义：包括区块数组，当前区块高度 */
pub struct Blockchain {
    blocks: Vec<Block>,
    height: usize,
}
      

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::create_genesis(CURR_BITS)],
            height: 0,
        }
    }

    /*
     * 挖矿示例：
     * 1. 遍历BlockHead中的nonce，以满足挖矿难度
     * 2. 生成一个新的区块并添加在当前区块链尾部
     */
    pub fn mining(&mut self, data: &str) {
        let pre_block = self.blocks.last().unwrap();
        let block = Block::new(data, pre_block.get_hash().as_str(), CURR_BITS);
        self.blocks.push(block);
        self.height += 1;
    }

    /* 打印区块链详细信息 */
    pub fn blocks_info(&self) {
        for block in self.blocks.iter() {
            info!("{:#?}", block);
        }
    }
}
      
/* 工作量证明机制 */
pub struct ProofOfWork {
    target: U256,   // 根据bits计算的工作难度，bigint库的U256类型
}

impl ProofOfWork {
    // 构造函数
    pub fn new(bits: usize) -> Self {
        // target初始化为1
        let mut target = U256::from(1 as usize);
        // left shift
        target = target.shl(256 - bits);

        Self {
            target
        }
    }

    // 根据传入的nonce，对区块头序列化
    fn prepare_data(block: &mut Block, nonce: usize) -> Result<Vec<u8>> {
        block.set_nonce(nonce);
        Ok(serialize(&(block.get_header()))?)
    }
      
    // 挖矿过程：遍历nonce寻找满足工作难度的hash
    pub fn run(&self, block: &mut Block) {
        let mut nonce = 0;

        while nonce < MAX_NONCE {
            if let Ok(iter_hash) = Self::prepare_data(block, nonce) {
                let mut hash_u: [u8; 32] = [0; 32];
                hash_to_u8(&iter_hash, &mut hash_u);
                let iter_hash_int = U256::from(hash_u);

                // 如果hash值小于target，则满足条件；否则进行下一轮计算
                if iter_hash_int.lt(&(self.target)) {
                    block.set_hash(hash_to_str(&iter_hash));
                    break;
                } else {
                    nonce += 1;
                }
            }
        }
    }
}

/* 错误信息 */
#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Serialize or Deserialize error")]
    SerializeError(#[from] Box<bincode::ErrorKind>),
}

/*
 * 区块链辅助函数定义
 */

/* 区块体数据序列化 */
pub fn serialize<T>(data: &T) -> Result<Vec<u8>, BlockchainError>
where
    T: Serialize + ?Sized
{
    Ok(bincode::serialize(data)?)
}

/* 区块体数据反序列化 */
#[allow(dead_code)]
pub fn deserialize<'a, T>(data: &'a [u8]) -> Result<T, BlockchainError>
where
    T: Deserialize<'a> + ?Sized
{
    Ok(bincode::deserialize(data)?)
}

/* 根据字节流计算hash值，用于计算一个区块的hash */
pub fn hash_to_str(data: &[u8]) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input(data);
    hasher.result_str()
}

/* 根据字节流计算hash值，用于计算一个区块的hash */
#[allow(dead_code)]
pub fn hash_to_u8(data: &[u8], out: &mut [u8]) {
    let mut hasher = Sha3::sha3_256();
    hasher.input(data);
    hasher.result(out)
}
