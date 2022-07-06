use std::env::current_dir;

use hungrytiger::{Blockchain, RocksDb};

fn main() {
    tracing_subscriber::fmt().init();

    let path = current_dir().unwrap().join("data");
    let mut bc = Blockchain::new(RocksDb::new(path));

    bc.mining("1st tx: Justin -> Bob 2 btc");
    bc.mining("2nd tx: Justin -> Bruce 2 btc");

    bc.blocks_info();
}
