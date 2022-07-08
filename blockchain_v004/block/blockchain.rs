/*
 * 版本：0.0.4
 * 一个用rust实现区块链的简单例子，只是说明区块链的原理
 * 1. 仅有单个节点
 * 2. 对象式编程
 * 3. PoW挖矿
 * 4. 持久化存储，使用RocksDb
 * 5. 模块化
 * 6. 增加：交易功能
 */
use tracing::info;
use std::{collections::HashMap, sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}}};

use crate::{Block, Transaction, TxOutput, RocksDb, KVStorage};


// 常量，工作量证明的难度
const CURR_BITS: usize = 2;

/* 区块链定义：包括区块数组，当前区块高度 */
pub struct Blockchain<T = RocksDb> {
    storage: Arc<T>,            // 区块链的存储
    tip: Arc<RwLock<String>>,   // 当前区块链最后一个区块的hash
    height: AtomicUsize,        // 当前区块链的高度
}

impl<T: KVStorage> Blockchain<T> {
    pub fn new(storage: Arc<T>, genesis_addr: &str) -> Self {
        // 如果db中已经存储了区块链，则加载到内存
        if let Ok(Some(tip)) = storage.get_tip() {
            let height = storage.get_height().unwrap();
            Self {
                storage,
                tip: Arc::new(RwLock::new(tip)),
                height: AtomicUsize::new(height.unwrap()),
            }
        } else {
            // 生成新的区块链
            let genesis_block = Block::create_genesis(CURR_BITS, genesis_addr);
            let hash = genesis_block.get_hash();
            storage.update_blocks(&hash, &genesis_block, 0 as usize);
            Self {
                storage,
                tip: Arc::new(RwLock::new(hash)),
                height: AtomicUsize::new(0),
            }
        }
    }

    /*
     * 挖矿示例：
     * 1. 遍历BlockHead中的nonce，以满足挖矿难度
     * 2. 生成一个新的区块并添加在当前区块链尾部
     */
    pub fn mining(&mut self, txs: &[Transaction]) {
        let block = Block::new(txs, &self.tip.read().unwrap(), CURR_BITS);
        let hash = block.get_hash();
        self.height.fetch_add(1, Ordering::Relaxed);
        self.storage.update_blocks(&hash, &block, self.height.load(Ordering::Relaxed));

        let mut tip = self.tip.write().unwrap();
        *tip = hash;
    }

    fn get_blocks(&self) -> Vec<Block> {
        let tip = &self.tip.read().unwrap();
        let last_block = &self.storage.get_block(tip).unwrap().unwrap();

        let mut blocks: Vec<Block> = vec![(*last_block).clone()];

        let mut hash = last_block.get_prev_hash();
        while !hash.is_empty() {
            let block = &self.storage.get_block(hash.as_str()).unwrap().unwrap();
            hash = block.get_prev_hash();
            blocks.push((*block).clone());
        }

        blocks
    }

    pub fn find_utxo(&self) -> HashMap<String, Vec<TxOutput>> {
        let mut utxo = HashMap::new();
        let mut spent_txos = HashMap::new();

        let mut blocks = self.get_blocks();
        while let Some(block) = blocks.pop() {
            for tx in block.get_transactions() {
                for (idx, tx_out) in tx.get_vout().iter().enumerate() {
                    if let Some(outs) = spent_txos.get(&tx.get_id()) {
                        for out in outs {
                            if idx.eq(out) {
                                break;
                            }

                            utxo.entry(tx.get_id())
                                .and_modify(|v: &mut Vec<TxOutput>| v.push(tx_out.clone()))
                                .or_insert(vec![tx_out.clone()]);
                        }
                    } else {
                        utxo.entry(tx.get_id())
                            .and_modify(|v: &mut Vec<TxOutput>| v.push(tx_out.clone()))
                            .or_insert(vec![tx_out.clone()]);
                    }
                }

                for tx_in in tx.get_vin() {
                    spent_txos.entry(tx_in.get_txid())
                        .and_modify(|v: &mut Vec<usize>| v.push(tx_in.get_vout()))
                        .or_insert(vec![tx_in.get_vout()]);
                }
            }
        }

        utxo
    }

    /* 打印区块链详细信息 */
    pub fn blocks_info(&self) {
        let mut blocks = self.get_blocks();
        while let Some(block) = blocks.pop() {
            info!("{:#?}", block);
        }
    }
}
