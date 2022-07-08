use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/*
 * 交易输入数据结构
 * 基于UTXO的每一笔交易都包含输入和输出，其中输入必须是
 * 某个未花费地址的余额
 */
pub struct TxInput {
    txid: String,   // 前一笔交易的id
    vout: usize,    // 前一笔交易的输出序号
    from_addr: String, // 交易发起方地址
}

impl TxInput {
    pub fn new(txid: String, vout: usize, from_addr: &str) -> Self {
        Self {
            txid,
            vout,
            from_addr: from_addr.into(),
        }
    }

    // 判断交易发起方同接收方地址是否相同
    pub fn is_unlock_output(&self, output_addr: &str) -> bool {
        self.from_addr.eq(output_addr)
    }

    pub fn get_txid(&self) -> String {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }
}
