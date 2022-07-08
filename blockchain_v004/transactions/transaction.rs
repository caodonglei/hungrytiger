use serde::{Serialize, Deserialize};

use crate::{TxInput, TxOutput, utils::{serialize, hash_to_str}, UTXOSet, KVStorage};

// 挖矿的奖励，20枚代币
const SUBSIDY: i32 = 20;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/*
 * 交易定义
 */
pub struct Transaction {
    id: String,             // 该笔交易的id
    vin: Vec<TxInput>,      // 交易输入集合，即交易发起方所有可花费账户
    vout: Vec<TxOutput>,    // 交易输出集合，即因交易产生的新的可花费账户
}

impl Transaction {
    // 新区块的奖励，没有输入地址，to是矿工地址
    pub fn new_coinbase(to: &str) -> Self {
        let tx_in = TxInput::default();
        let tx_out = TxOutput::new(SUBSIDY, to);

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![tx_in],
            vout: vec![tx_out],
        };

        tx.set_hash();
        tx
    }

    // 新的UTXO转账，从from地址转账给to地址，共amount枚代币
    // 必须检查from地址的代币没有被消费过
    pub fn new_utxo_transaction<T: KVStorage>(from: &str, to: &str, amount: i32, utxo_set: &UTXOSet<T>) -> Self {
        // 查询utxo集合中from地址的可花费账户余额总和accumulated
        // valid_outputs是from地址可以花费的所有账号
        let (accumulated, valid_outputs) = utxo_set.find_spendable_outputs(from, amount);
        if accumulated < amount {
            panic!("Error not enough funds");
        }

        // 本次交易使用掉的utxo账户
        let mut inputs = vec![];
        for (txid, outputs) in valid_outputs {
            for idx in outputs {
                let input = TxInput::new(txid.clone(), idx.clone(), from);
                inputs.push(input);
            }
        }

        // 本次交易新生成的utxo账户
        let mut outputs = vec![TxOutput::new(amount, &to)];
        // 这是对交易发起方的utxo找零账户
        if accumulated > amount {
            outputs.push(TxOutput::new(accumulated - amount, &from));
        }

        let mut tx = Transaction {
            id: String::new(),
            vin: inputs,
            vout: outputs,
        };
        // 生成新的交易id
        tx.set_hash();

        tx
    }

    // 对交易本身签名
    pub fn set_hash(&mut self) {
        if let Ok(tx_ser) = serialize(self) {
            self.id = hash_to_str(&tx_ser)
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_vin(&self) -> &[TxInput] {
        self.vin.as_slice()
    }

    pub fn get_vout(&self) -> &[TxOutput] {
        self.vout.as_slice()
    }
}
