fn main() {
    tracing_subscriber::fmt().init();

    let mut bc = hungrytiger::create_blockchain();

    hungrytiger::mining(&mut bc, "1st tx: Justin -> Bob 2 btc");
    hungrytiger::mining(&mut bc, "2nd tx: Justin -> Bruce 2 btc");

    hungrytiger::blocks_info(&bc);
}
