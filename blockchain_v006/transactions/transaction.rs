use serde::{Serialize, Deserialize};

use crate::{TxInput, TxOutput, utils::{serialize, hash_to_str, ecdsa_p256_sha256_sign_digest, ecdsa_p256_sha256_sign_verify}, UTXOSet, KVStorage, Wallets, hash_pub_key, Blockchain};


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
    pub fn new_utxo_transaction<T: KVStorage>(from: &str, to: &str, amount: i32, utxo_set: &UTXOSet<T>, bc: &Blockchain<T>) -> Self {
        let wallets = Wallets::new().unwrap();
        let wallet = wallets.get_wallet(from).unwrap();
        let public_key_hash = hash_pub_key(wallet.get_public_key());

        // 查询utxo集合中from地址的可花费账户余额总和accumulated
        // valid_outputs是from地址可以花费的所有账号
        let (accumulated, valid_outputs) = utxo_set.find_spendable_outputs(&public_key_hash, amount);
        if accumulated < amount {
            panic!("Error not enough funds");
        }

        // 本次交易使用掉的utxo账户
        let mut inputs = vec![];
        for (txid, outputs) in valid_outputs {
            for idx in outputs {
                let input = TxInput::new(txid.clone(), idx.clone(), wallet.get_public_key().to_vec());
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
        tx.sign(bc, wallet.get_private_key());

        tx
    }

    // 对交易本身签名
    pub fn set_hash(&mut self) {
        if let Ok(tx_ser) = serialize(self) {
            self.id = hash_to_str(&tx_ser)
        }
    }

    fn sign<T: KVStorage>(&mut self, bc: &Blockchain<T>, private_key: &[u8]) {
        let mut tx_copy = self.trimmed_copy();

        for (idx, vin) in self.vin.iter_mut().enumerate() {
            // 查找输入引用的交易
            let prev_tx_option = bc.find_transaction(vin.get_txid());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[idx].set_signature(vec![]);
            tx_copy.vin[idx].set_pub_key(prev_tx.vout[vin.get_vout()].get_pub_key_hash());
            tx_copy.set_hash();

            tx_copy.vin[idx].set_pub_key(&vec![]);

            // 使用私钥对数据签名
            let signature = ecdsa_p256_sha256_sign_digest(private_key, tx_copy.id.as_bytes());
            vin.set_signature(signature);
        }
    }

    pub fn verify<T: KVStorage>(&self, bc: &Blockchain<T>) -> bool {
        if self.is_coinbase() {
            return true;
        }

        let mut tx_copy = self.trimmed_copy();
        for (idx, vin) in self.vin.iter().enumerate() {
            let prev_tx_option = bc.find_transaction(vin.get_txid());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[idx].set_signature(vec![]);
            tx_copy.vin[idx].set_pub_key(prev_tx.vout[vin.get_vout()].get_pub_key_hash());
            tx_copy.set_hash();

            tx_copy.vin[idx].set_pub_key(&vec![]);

            // 使用公钥验证签名
            let verify = ecdsa_p256_sha256_sign_verify(
                vin.get_pub_key(),
                vin.get_signature(),
                tx_copy.id.as_bytes(),
            );
            if !verify {
                return false;
            }
        }
        true
    }

    // 判断是否是 coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].get_pub_key().len() == 0
    }

    // 清空TxInput的signature，复制transaction
    fn trimmed_copy(&self) -> Transaction {
        let mut inputs = vec![];
        let mut outputs = vec![];
        for input in &self.vin {
            let txinput = TxInput::new(input.get_txid(), input.get_vout(), vec![]);
            inputs.push(txinput);
        }
        for output in &self.vout {
            outputs.push(output.clone());
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
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
