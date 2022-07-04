use hungrytiger::Blockchain;

fn main() {
    tracing_subscriber::fmt().init();

    let mut bc = Blockchain::new();

    bc.mining("1st tx: Justin -> Bob 2 btc");
    bc.mining("2nd tx: Justin -> Bruce 2 btc");

    bc.blocks_info();
}
