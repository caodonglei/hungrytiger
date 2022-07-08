use serde::{Serialize, Deserialize};

// 交易输出数据结构，这是一个新的UTXO账户
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxOutput {
    value: i32,         // 账户余额
    to_addr: String,    // 账户地址
}

impl TxOutput {
    pub fn new(value: i32, to_addr: &str) -> Self {
        Self {
            value,
            to_addr: to_addr.into(),
        }
    }

    // 判断address地址是否有拥有该输出
    pub fn is_locked(&self, address: &str) -> bool {
        self.to_addr.eq(address)
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}
