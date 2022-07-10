use std::{env::current_dir, sync::Arc};

use hungrytiger::{Blockchain, RocksDb, UTXOSet, Transaction, Wallets};

/* 创建区块链 */
fn generate_blockchain() -> String {
    tracing_subscriber::fmt().init();

    // 区块链持久化存储路径
    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    // 创世块的矿工地址
    let mut wallets = Wallets::new().unwrap();
    let genesis_addr = wallets.create_wallet();
    println!("==> genesis address: {}", genesis_addr);

    let bc = Blockchain::new(storage.clone(), &genesis_addr);
    let utxos = UTXOSet::new(storage);
    utxos.reindex(&bc).unwrap();

    bc.blocks_info();

    genesis_addr
}

/* 创建交易 */
fn create_transactions(caodl_addr: String) {
    let mut wallets = Wallets::new().unwrap();
    let bob_addr = wallets.create_wallet();
    let alice_addr = wallets.create_wallet();

    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    let mut bc = Blockchain::new(storage.clone(), caodl_addr.as_str());
    let utxos = UTXOSet::new(storage);

    let tx1 = Transaction::new_utxo_transaction(
        caodl_addr.as_str(), bob_addr.as_str(), 3, &utxos, &bc);
    let tx2 = Transaction::new_utxo_transaction(
        caodl_addr.as_str(), alice_addr.as_str(), 2, &utxos, &bc);

    let txs = vec![tx1, tx2];
    bc.mining(&txs);
    utxos.reindex(&bc).unwrap();

    bc.blocks_info();
}


fn main() {
    let caodl_addr = generate_blockchain();
    create_transactions(caodl_addr);
}
