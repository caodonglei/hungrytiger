use std::{env::current_dir, sync::Arc};

use hungrytiger::{Blockchain, RocksDb, UTXOSet, Transaction, Wallets};

/* 创建区块链 */
fn generate_blockchain() -> String {
    tracing_subscriber::fmt().init();

    // 区块链持久化存储路径
    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    // 创世块的矿工地址
    //let mut wallets = Wallets::new().unwrap();
    //let genesis_addr = wallets.create_wallet();
    let genesis_addr: String = String::from("1GMXdoTqE4wfG1kdxDDkYz4qNr7x3dMG8b");
    println!("==> genesis address: {}", genesis_addr);

    let mut bc = Blockchain::new(storage.clone());
    bc.create_genesis_block(&genesis_addr);

    let utxos = UTXOSet::new(storage);
    utxos.reindex(&bc).unwrap(); 

    bc.blocks_info();

    genesis_addr
}

/* 创建交易 */
fn create_transactions(caodl_addr: String) {
    let mut wallets = Wallets::new().unwrap();
    //let bob_addr = wallets.create_wallet();
    //let alice_addr = wallets.create_wallet();
    let bob_addr: String = String::from("12myUvFZp3zVzQoQcT7tmWWPR49jeiXjfJ");
    let alice_addr: String = String::from("1ShUqhwCP43ZLAoiPEBcZLSeJVuFytW7F");


    let path = current_dir().unwrap().join("data");
    let storage = Arc::new(RocksDb::new(path));

    let mut bc = Blockchain::new(storage.clone());
    let utxos = UTXOSet::new(storage);

    let tx1 = Transaction::new_utxo_transaction(
        caodl_addr.as_str(), bob_addr.as_str(), 3, &utxos, &bc);
    let tx2 = Transaction::new_utxo_transaction(
        caodl_addr.as_str(), alice_addr.as_str(), 2, &utxos, &bc);

    let txs = vec![tx1, tx2];
    bc.mining(&txs);
    utxos.reindex(&bc).unwrap();

    bc.blocks_info();

    let mut pubkey = wallets.get_wallet(caodl_addr.as_str()).unwrap().get_public_key();
    let mut balance = utxos.get_balance(pubkey);
    println!("{}'s blance={}", caodl_addr, balance);

    pubkey = wallets.get_wallet(bob_addr.as_str()).unwrap().get_public_key();
    balance = utxos.get_balance(pubkey);
    println!("{}'s blance={}", bob_addr, balance);

    pubkey = wallets.get_wallet(alice_addr.as_str()).unwrap().get_public_key();
    balance = utxos.get_balance(pubkey);
    println!("{}'s blance={}", alice_addr, balance);
}


fn main() {
    let caodl_addr = generate_blockchain();
    create_transactions(caodl_addr);
}
