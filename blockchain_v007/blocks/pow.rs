use anyhow::Result;
use std::ops::Shl;
use bigint::U256;

use crate::{utils::{serialize, hash_to_u8, hash_to_str}, Block};

// 挖矿最大迭代次数
const MAX_NONCE: usize = usize::MAX;

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

