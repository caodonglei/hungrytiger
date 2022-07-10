use serde::{Serialize, Deserialize};

use crate::{utils::base58_decode, ADDRESS_CHECKSUM_LEN};

// 交易输出数据结构，这是一个新的UTXO账户
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxOutput {
    value: i32,         // 账户余额
    pub_key_hash: Vec<u8>,    // 账户公钥hash
}

impl TxOutput {
    pub fn new(value: i32, to_addr: &str) -> Self {
        let mut output = TxOutput {
            value,
            pub_key_hash: vec![],
        };
        output.lock(to_addr);
        output
    }

    // 从地址中截取出public key hash值
    fn lock(&mut self, address: &str) {
        let payload = base58_decode(address);
        let pub_key_hash = payload[1..payload.len() - ADDRESS_CHECKSUM_LEN].to_vec();
        self.pub_key_hash = pub_key_hash
    }

    // 判断address地址是否有拥有该输出
    pub fn is_locked(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_pub_key_hash(&self) -> &[u8] {
        self.pub_key_hash.as_slice()
    }
}
