use std::{path::Path, collections::HashMap};
use std::sync::Arc;
use rocksdb::{{DB, IteratorMode}};

use crate::{Block, KVStorage, error::BlockchainError, utils::{deserialize, serialize}, TIP_KEY, HEIGHT, TABLE_OF_BLOCK, UTXO_SET, TxOutput};

/*
 * 数据库实现
 */
#[derive(Clone)]
pub struct RocksDb {
    db: Arc<DB>,    // KV storage
}

impl RocksDb {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            db: Arc::new(DB::open_default(path).unwrap())
        }
    }

    // 将table + block hash共同拼成一个key
    fn get_full_key(table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }
}

impl KVStorage for RocksDb {
    // 查询数据库中，区块链最后一个block的hash
    fn get_tip(&self) -> Result<Option<String>, BlockchainError> {
        let result = self.db.get(TIP_KEY)?.map(|v| deserialize::<String>(&v.to_vec()));
        result.map_or(Ok(None), |v| v.map(Some))
    }

    fn get_block(&self, key: &str) -> Result<Option<Block>, BlockchainError> {
        let name = Self::get_full_key(TABLE_OF_BLOCK, key);
        let result = self.db.get(name)?.map(|v| v.into());
        Ok(result)
    }

    fn get_height(&self) -> Result<Option<usize>, BlockchainError> {
        let result = self.db.get(HEIGHT)?.map(|v| deserialize::<usize>(&v.to_vec()));
        result.map_or(Ok(None), |v| v.map(Some))
    }

    // 存储新的block，以该block的hash为key
    fn update_blocks(&self, key: &str, block: &Block, height: usize) {
        //let _: TransactionResult<(), ()> = self.db.transaction(|db| {
        let row_key = Self::get_full_key(TABLE_OF_BLOCK, key);
        self.db.put(row_key.as_str(), serialize(block).unwrap());
        self.db.put(TIP_KEY, serialize(key).unwrap());
        self.db.put(HEIGHT, serialize(&height).unwrap());
        self.db.flush();
            //Ok(())
        //});
    }

    fn get_utxo_set(&self) -> HashMap<String, Vec<TxOutput>> {
        let mut map = HashMap::new();

        let prefix = format!("{}:", UTXO_SET);

        let iter = self.db.iterator(IteratorMode::Start); // Always iterates forward
        for (k, v) in iter {
            if k.starts_with(prefix.as_bytes()) {
                let txid = String::from_utf8(k.to_vec()).unwrap();
                let txid = txid.split(":").collect::<Vec<_>>()[1].into();
                let outputs = deserialize::<Vec<TxOutput>>(&v.to_vec()).unwrap();

                map.insert(txid, outputs);
            }
        }
        map
    }

    // 写入utxo账户，基于前缀UTXO_SET
    fn write_utxo(&self, txid: &str, outs: Vec<TxOutput>) -> Result<(), BlockchainError> {
        let name = format!("{}:{}", UTXO_SET, txid);
        self.db.put(name, serialize(&outs)?)?;
        Ok(())
    }


    // 清空utxo集合
    fn clear_utxo_set(&self) {
        let prefix = format!("{}:", UTXO_SET);
        self.db.delete(prefix).unwrap();
    }
}

impl From<Vec<u8>> for Block {
    fn from(v: Vec<u8>) -> Self {
        let result = deserialize::<Block>(&v.to_vec());
        match result {
            Ok(block) => block,
            Err(_) => Block::default(),
        }
    }
}
