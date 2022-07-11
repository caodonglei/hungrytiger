use serde::{Serialize, Deserialize};

use crate::hash_pub_key;

/*
 * 交易输入数据结构
 * 基于UTXO的每一笔交易都包含输入和输出，其中输入必须是
 * 某个未花费地址的余额
 */
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxInput {
    txid: String,   // 前一笔交易的id
    vout: usize,    // 前一笔交易的输出自增序号
    signature: Vec<u8>, // 交易发起方使用私钥对交易的签名
    pub_key: Vec<u8>,   // 交易发起方的公钥
}

impl TxInput {
    pub fn new(txid: String, vout: usize, pub_key: Vec<u8>) -> Self {
        Self {
            txid,
            vout,
            signature: vec![],
            pub_key,
        }
    }

    // 判断交易发起方同签名是否一致
    pub fn is_unlock_output(&self, pub_key_hash: &[u8]) -> bool {
        let locked_hash = hash_pub_key(&self.pub_key);
        locked_hash.eq(pub_key_hash)
    }

    pub fn get_txid(&self) -> String {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }
