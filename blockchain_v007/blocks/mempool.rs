use std::collections::HashMap;
use std::sync::RwLock;

use crate::Transaction;

/// 交易内存池 ( K -> txid, V => Transaction )
pub struct MemoryPool {
    inner: RwLock<HashMap<String, Transaction>>,
}

impl MemoryPool {
    pub fn new() -> MemoryPool {
        MemoryPool {
            inner: RwLock::new(HashMap::new()),
        }
    }

    // 根据txid，判断mempool中是否包含该笔交易
    pub fn containes(&self, txid: &str) -> bool {
        self.inner.read().unwrap().contains_key(txid)
    }

    // 将一笔交易加入到mempool
    pub fn add(&self, tx: Transaction) {
        let txid = tx.get_id();
        self.inner.write().unwrap().insert(txid, tx);
    }

    // 根据txid获取mempool中的一笔交易
    pub fn get(&self, txid: &str) -> Option<Transaction> {
        if let Some(tx) = self.inner.read().unwrap().get(txid) {
            return Some(tx.clone());
        }
        None
    }

    // 根据txid删除一笔交易
    pub fn remove(&self, txid_hex: &str) {
        let mut inner = self.inner.write().unwrap();
        inner.remove(txid_hex);
    }

    // 获取mempool中所有交易
    pub fn get_all(&self) -> Vec<Transaction> {
        let inner = self.inner.read().unwrap();
        let mut txs = vec![];
        for (_, v) in inner.iter() {
            txs.push(v.clone());
        }
        return txs;
    }

    // 返回mempool当前存储的交易数量
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

// 传输中的块
pub struct BlockInTransit {
    inner: RwLock<Vec<Vec<u8>>>,
}

impl BlockInTransit {
    pub fn new() -> BlockInTransit {
        BlockInTransit {
            inner: RwLock::new(vec![]),
        }
    }

    pub fn add_blocks(&self, blocks: &[Vec<u8>]) {
        let mut inner = self.inner.write().unwrap();
        for hash in blocks {
            inner.push(hash.to_vec());
        }
    }

    pub fn first(&self) -> Option<Vec<u8>> {
        let inner = self.inner.read().unwrap();
        if let Some(block_hash) = inner.first() {
            return Some(block_hash.to_vec());
        }
        None
    }

    pub fn remove(&self, block_hash: &[u8]) {
        let mut inner = self.inner.write().unwrap();
        if let Some(idx) = inner.iter().position(|x| x.eq(block_hash)) {
            inner.remove(idx);
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}
