/*
 * 版本：0.0.7
 * 一个用rust实现区块链的简单例子，只是说明区块链的原理
 * 1. 对象式编程
 * 2. PoW挖矿
 * 3. 持久化存储，使用RocksDb
 * 4. 模块化
 * 5. UTXO账户交易功能
 * 6. 钱包
 * 7. P2P网络功能，基于libp2p类库
 * 8. 增加mempool，并修复若干bugs
 */
use std::{collections::HashMap, sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}}};

use crate::{Block, Transaction, TxOutput, RocksDb, KVStorage, error::BlockchainError};


// 常量，工作量证明的难度
const CURR_BITS: usize = 2;

/* 区块链定义：包括区块数组，当前区块高度 */
pub struct Blockchain<T = RocksDb> {
    storage: Arc<T>,            // 区块链的存储
    tip: Arc<RwLock<String>>,   // 当前区块链最后一个区块的hash
    height: AtomicUsize,        // 当前区块链的高度
}

impl<T: KVStorage> Blockchain<T> {
    pub fn new(storage: Arc<T>) -> Self {
        // 如果db中已经存储了区块链，则加载到内存
        if let Ok(Some(tip)) = storage.get_tip() {
            let height = storage.get_height().unwrap();
            Self {
                storage,
                tip: Arc::new(RwLock::new(tip)),
                height: AtomicUsize::new(height.unwrap()),
            }
        } else {
            Self {
                storage,
                tip: Arc::new(RwLock::new(String::new())),
                height: AtomicUsize::new(0),
            }
        }
    }

    // 生成创世块
    pub fn create_genesis_block(&mut self, genesis_addr: &str) {
        let genesis_block = Block::create_genesis(CURR_BITS, genesis_addr);
        let hash = genesis_block.get_hash();
        self.height.fetch_add(1, Ordering::Relaxed);
        self.storage.update_blocks(&hash, &genesis_block, self.height.load(Ordering::Relaxed));
        let mut tip = self.tip.write().unwrap();
        *tip = hash;
    }

    /* 
     * 挖矿示例：
     * 1. 遍历BlockHead中的nonce，以满足挖矿难度
     * 2. 生成一个新的区块并添加在当前区块链尾部 
     */
    pub fn mining(&mut self, txs: &[Transaction]) -> Block {
        // 逐笔验证交易
        for tx in txs {
            if tx.verify(self) == false {
                panic!("ERROR: Invalid transaction")
            }
        }
        let block = Block::new(txs, &self.tip.read().unwrap(), CURR_BITS);
        let hash = block.get_hash();
        self.height.fetch_add(1, Ordering::Relaxed);
        self.storage.update_blocks(&hash, &block, self.height.load(Ordering::Relaxed));

        let mut tip = self.tip.write().unwrap();
        *tip = hash;

        block
    }

    // 在当前区块链尾部添加新块，这个块是由其他节点挖到的
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        let hash = block.get_hash();
        if let Some(_) = self.storage.get_block(&hash)? {
            println!("Block {} already exists", hash);
        } else {
            self.height.fetch_add(1, Ordering::Relaxed);
            self.storage.update_blocks(&hash, &block, self.height.load(Ordering::Relaxed));
            let mut tip = self.tip.write().unwrap();
            *tip = hash;
        }
        Ok(())
    }

    /* 遍历所有区块，栈结构，使用pop方法从第一个区块开始遍历 */
    fn get_blocks(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = vec![];
        let mut iter = BlockchainIterator::new(self.get_tip(), self.storage.clone());
        loop {
            let block = iter.next();
            if block.is_none() {
                break;
            }
            blocks.push(block.unwrap().clone());
        }
    
        blocks
    }

    // 导出顺序排列的区块
    pub fn dump_blocks(&self) -> Vec<Block> {
        let mut ordered_blocks = vec![];
        let mut blocks = self.get_blocks();
        while let Some(block) = blocks.pop() {
            ordered_blocks.push(block);
        }
        ordered_blocks
    }

    /* 遍历区块链，找到所有维护费交易UTXO  */
    pub fn find_utxo(&self) -> HashMap<String, Vec<TxOutput>> {
        let mut utxo = HashMap::new();
        let mut spent_txos = HashMap::new();

        // 遍历区块链，必须倒序！
        let mut iter = BlockchainIterator::new(self.get_tip(), self.storage.clone());
        loop {
            let block = iter.next();
            if block.is_none() {
                break;
            }

            // 遍历当前区块的所有交易
            for tx in block.unwrap().get_transactions() {
                // 遍历当前交易的所有输出
                for (idx, tx_out) in tx.get_vout().iter().enumerate() {
                    // 如果已花费账户map中已经记录了transaction id，则需要
                    // 判断当前输出是否已经被花费
                    if let Some(spent_outs) = spent_txos.get(&tx.get_id()) {
                        for out in spent_outs {
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

                // 将当前交易的inputs加入到已消费账户中，一个TxInput必然是之前某个
                // 交易的TxOutput，用(txid, vout)唯一定义
                for tx_in in tx.get_vin() {
                    spent_txos.entry(tx_in.get_txid())
                        .and_modify(|v: &mut Vec<usize>| v.push(tx_in.get_vout()))
                        .or_insert(vec![tx_in.get_vout()]);
                }
            }
        }

        utxo
    }
    
    // 根据transaction id在所有block中查找该笔交易
    pub fn find_transaction(&self, txid: String) -> Option<Transaction> {
        let mut iter = BlockchainIterator::new(self.get_tip(), self.storage.clone());
        loop {
            let block = iter.next();
            if block.is_none() {
                break;
            }

            for tx in block.unwrap().get_transactions() {
                if tx.get_id() == txid {
                    return Some(tx);
                }
            }
        }
        None
    }

    /* 打印区块链详细信息 */
    pub fn blocks_info(&self) {
        let mut blocks = self.get_blocks();
        while let Some(block) = blocks.pop() {
            println!("{:#?}", block);
        }
    }

    pub fn get_tip(&self) -> String {
        self.tip.read().unwrap().to_string()
    }

    pub fn get_height(&self) -> usize {
        self.height.load(Ordering::Relaxed)
    }
}


/* 以倒序遍历全部区块 */
pub struct BlockchainIterator<T = RocksDb> {
    storage: Arc<T>,
    next_block_hash: String,     // 下一个区块的hash值
}

impl<T: KVStorage> BlockchainIterator<T> {
    #[warn(dead_code)]
    fn new(tip: String, storage: Arc<T>) -> Self {
        Self {
            storage,
            next_block_hash: tip,
        }
    }

    pub fn next(&mut self) -> Option<Block> {
        let block_hash = &self.next_block_hash;
        if let Some(block) = &self.storage.get_block(block_hash.as_str()).unwrap() {
            self.next_block_hash = block.get_prev_hash().clone();
            
            Some(block.clone())
        } else {
            None
        }
    }
}
