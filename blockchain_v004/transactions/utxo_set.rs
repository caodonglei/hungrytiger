use std::{collections::HashMap, sync::Arc};

use crate::{KVStorage, Blockchain, error::BlockchainError};

/* 可花费交易账户集合 */
pub struct UTXOSet<T> {
    storage: Arc<T>
}

impl<T: KVStorage> UTXOSet<T> {
    pub fn new(storage: Arc<T>) -> Self {
        Self {
            storage
        }
    }

    // 每次交易之后，需要清空utxo集合，并且重新生成utxo集合
    pub fn reindex(&self, bc: &Blockchain) -> Result<(), BlockchainError> {
        self.storage.clear_utxo_set();
        let map = bc.find_utxo();
        for (txid, outs) in map {
            // 将新生成的utxo账户写入数据库
            self.storage.write_utxo(&txid, outs)?;
        }
        Ok(())
    }

    // 查找from_addr地址拥有的所有未花费账户
    pub fn find_spendable_outputs(&self, from_addr: &str, amount: i32) -> (i32, HashMap<String, Vec<usize>>) {
        // 所有未花费账户存储在hashmap中
        let mut unspent_outpus = HashMap::new();
        // from_addr所有未花费账户的余额总和
        let mut accumulated = 0;
        let utxo_set = self.storage.get_utxo_set();

        for (txid, outs) in utxo_set.iter() {
            for (idx, out) in outs.iter().enumerate() {
                if out.is_locked(from_addr) && accumulated < amount {
                    accumulated += out.get_value();
                    unspent_outpus.entry(txid.to_string()).and_modify(|v: &mut Vec<usize>| v.push(idx)).or_insert(vec![idx]);
                }
            }
        }

        (accumulated, unspent_outpus)
    }
}
