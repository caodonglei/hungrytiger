use std::{collections::HashMap, env::current_dir, fs};

use serde::{Serialize, Deserialize};
use tracing::info;

use crate::{Wallet, utils::{serialize, deserialize}, error::BlockchainError};

// 将wallet数据结构保存在本地wallet.dat文件中
pub const WALLET_FILE: &str = "wallet.dat";

// 从address到wallet的映射
#[derive(Serialize, Deserialize)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Result<Self, BlockchainError> {
        let wallets = Self::load_wallet_from_file();
        wallets
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        self.save_wallet_to_file().unwrap();
        address
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    pub fn get_addresses(&self) -> Vec<&String> {
        self.wallets.keys().collect()
    }

    // 将wallet数据存储在本地文件
    pub fn save_wallet_to_file(&self) -> Result<(), BlockchainError> {
        let path = current_dir().unwrap().join(WALLET_FILE);
        let wallets_ser = serialize(&self)?;
        fs::write(path, &wallets_ser).unwrap();
        Ok(())
    }

    // 从本地文件加载wallet数据
    pub fn load_wallet_from_file() -> Result<Self, BlockchainError> {
        let path = current_dir().unwrap().join(WALLET_FILE);
        info!("Wallet path: {:?}", path);
        // 如果本地没有任何wallet，则创建一个空的hashmap
        if !path.exists() {
            let wallets = Wallets {
                wallets: HashMap::new(),
            };
            return Ok(wallets);
        }

        let wallets_ser = fs::read(&path).unwrap();
        let wallets = deserialize(&wallets_ser);
        wallets
    }
}
