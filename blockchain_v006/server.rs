use std::{env::{current_dir, self}, sync::Arc};

use anyhow::Result;
use hungrytiger::{Node, RocksDb};

/* 以server方式启动节点，监听其他节点的消息 */
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 第二个参数是区块链存储路径，默认为data目录
    let mut path = String::from("data");
    if let Some(args) = env::args().nth(2) {
        path = args;
    }

    let path = current_dir().unwrap().join(path);
    let db = Arc::new(RocksDb::new(path));
    let mut node = Node::new(db).await?;
    node.start().await?;
    Ok(())
}
