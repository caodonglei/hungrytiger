use std::{env::current_dir, sync::Arc};

use hungrytiger::{Blockchain, RocksDb, UTXOSet, Transaction};

/* 创建区块链 */
fn generate_blockchain() {
    tracing_subscriber::fmt().init();

    // 区块链持久化存储路径
    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    // 创世块的矿工地址
    let genesis_addr = "caodl";
    let bc = Blockchain::new(storage.clone(), genesis_addr);
    let utxos = UTXOSet::new(storage);
    utxos.reindex(&bc).unwrap();

    bc.blocks_info();
}

/* 创建交易 */
fn create_transactions() {
    let caodl_addr = "caodl";
    let bob_addr = "Bob";
    let alice_addr = "Alice";

    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    let mut bc = Blockchain::new(storage.clone(), caodl_addr);
    let utxos = UTXOSet::new(storage);

    let tx1 = Transaction::new_utxo_transaction(caodl_addr, bob_addr, 3, &utxos);
    let tx2 = Transaction::new_utxo_transaction(caodl_addr, alice_addr, 2, &utxos);

    let txs = vec![tx1, tx2];
    bc.mining(&txs);
    utxos.reindex(&bc).unwrap();

    bc.blocks_info();
}


fn main() {
    generate_blockchain();
    create_transactions();
}
