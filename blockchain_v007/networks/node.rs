use std::sync::Arc;
use once_cell::sync::Lazy;
use futures::StreamExt;
use libp2p::{Swarm, swarm::SwarmEvent, PeerId};
use anyhow::Result;
use tokio::{
    io::{BufReader, stdin, AsyncBufReadExt}, 
    sync::mpsc
};
use tracing::{error};
use crate::{Blockchain, BlockchainBehaviour, KVStorage, RocksDb, UTXOSet, Commands, Messages, Block, Wallets, Transaction, MemoryPool};

use super::{create_swarm, BLOCK_TOPIC, TRANX_TOPIC, PEER_ID, WALLET_MAP};

/* 内存池中的交易到达阈值, 触发矿工挖新区块 */
pub const TRANSACTION_THRESHOLD: usize = 4;
/* 交易内存池 */
static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool::new());

// 本机矿工地址，后续可以改为配置文件
static MINER_ADDRESS: &str = "1GMXdoTqE4wfG1kdxDDkYz4qNr7x3dMG8b";

// 增加网络功能后，不再new blockchain，而是仅new一个node
pub struct Node<T = RocksDb> {
    bc: Blockchain<T>,      // 每个node包含一个本地区块链
    utxos: UTXOSet<T>,      // 每个node包含一个未消费账户集合
    msg_receiver: mpsc::UnboundedReceiver<Messages>,    // 消息接收端
    swarm: Swarm<BlockchainBehaviour>,      // rust-libp2p的swarm
}

impl<T: KVStorage> Node<T> {
    // 异步构造函数
    pub async fn new(storage: Arc<T>) -> Result<Self> {
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            bc: Blockchain::new(storage.clone()),
            utxos: UTXOSet::new(storage),
            msg_receiver,
            swarm: create_swarm(vec![BLOCK_TOPIC.clone(), TRANX_TOPIC.clone()], msg_sender).await?,
        })
    }

    // 列出所有节点
    pub async fn list_peers(&mut self) -> Result<Vec<&PeerId>> {
        // 基于MDNS协议发现网络中的节点
        let nodes = self.swarm.behaviour().mdns.discovered_nodes();
        let peers = nodes.collect::<Vec<_>>();
        Ok(peers)
    }

    // 同步本节点的区块高度，通过广播
    async fn sync(&mut self) -> Result<()> {
        // 封装本节点的区块高度
        let version = Messages::Version { 
            best_height: self.bc.get_height(), 
            from_addr: PEER_ID.to_string(),
        };
                    
        let line = serde_json::to_vec(&version)?;
        // 广播该消息，通过BLOCK_TOPIC channel
        self.swarm.behaviour_mut().gossipsub
            .publish(BLOCK_TOPIC.clone(), line).unwrap();
        Ok(())
    }

    // 异步转账并可能触发挖矿
    async fn transfer(&mut self, from: &str, to: &str, amount: i32) -> Result<()> {
        // 首先完成转账
        let tx = Transaction::new_utxo_transaction(from, to, amount, &self.utxos, &self.bc);
        // 将交易加入memory pool
        // TODO: 需要检查mempool中交易是否双花
        GLOBAL_MEMORY_POOL.add(tx);

        // 出新块
        if GLOBAL_MEMORY_POOL.len() >= TRANSACTION_THRESHOLD { 
            // coinbase transaction必须是block中第一条交易
            let coinbase_tx = Transaction::new_coinbase(MINER_ADDRESS);
            let mut txs = GLOBAL_MEMORY_POOL.get_all();
            txs.insert(0, coinbase_tx);

            let block = self.bc.mining(&txs);
            self.utxos.reindex(&self.bc).unwrap();

            // 从内存池中移除交易
            for tx in &txs[1..] {
                GLOBAL_MEMORY_POOL.remove(tx.get_id().as_str());
            }

            // 广播新块
            let block_store = Messages::Block { block };
            let line = serde_json::to_vec(&block_store)?;
            self.swarm.behaviour_mut().gossipsub
                .publish(BLOCK_TOPIC.clone(), line).unwrap();        
        }

        Ok(())
    }

    // 处理区块版本（高度），如果本节点区块高度更大，则广播整个区块链
    async fn process_version_msg(&mut self, best_height: usize, from_addr: String) -> Result<()> {
        if self.bc.get_height() > best_height {
            let blocks = Messages::Blocks { 
                blocks: self.bc.dump_blocks(),
                height: self.bc.get_height(),
                to_addr: from_addr,
            };
            let msg = serde_json::to_vec(&blocks)?;
            self.swarm.behaviour_mut().gossipsub
                .publish(BLOCK_TOPIC.clone(), msg).unwrap();
            }
        Ok(())
    }

    // 处理区块链，若接收到更长的区块链，则替换本地区块链
    async fn process_blocks_msg(&mut self, blocks: Vec<Block>, to_addr: String, height: usize) -> Result<()> {
        if PEER_ID.to_string() == to_addr && self.bc.get_height() < height {
            for block in blocks {
                self.bc.add_block(block)?;
            }

            self.utxos.reindex(&self.bc).unwrap();
        }
        Ok(())
    }

    // 处理区块，接收到其他节点挖到的新块
    pub async fn process_block_msg(&mut self, block: Block) -> Result<()> {
        self.bc.add_block(block)?;
        self.utxos.reindex(&self.bc).unwrap();
        Ok(())
    }

    // 启动服务，入口函数，主逻辑
    pub async fn start(&mut self) -> Result<()> {
        // 监听端口
        self.swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;
                
        // 监听命令行输入
        let mut stdin = BufReader::new(stdin()).lines();
                
        loop {
            tokio::select! {
                // line是命令行输入
                line = stdin.next_line() => { 
                    let line = line?.expect("stdin closed");
                    let command = serde_json::from_str(line.as_str());
                    // Commands是个enum类型
                    match command {
                        Ok(cmd) => match cmd {
                            // 创世块命令，当前区块链必须为空
                            Commands::Genesis(addr) => {
                                if self.bc.get_tip().is_empty() {
                                    self.bc.create_genesis_block(addr.as_str());
                                    self.utxos.reindex(&self.bc)?;
                                    println!("Genesis block was created success!");
                                } else {
                                    println!("Already exists blockchain, don't need genesis block!");
                                    continue;
                                }
                            },
                            // 输出区块链状态
                            Commands::Blocks(_) => {
                                self.bc.blocks_info();
                                println!("tip: {}", self.bc.get_tip());
                                println!("height: {}", self.bc.get_height());
                            },
                            // 同步区块链状态
                            Commands::Sync(_) => {
                               self.sync().await?;
                            },
                            // 生成钱包
                            Commands::CreateWallet(name) => {
                                WALLET_MAP.lock().await.entry(name.clone())
                                    .or_insert_with(|| {
                                    let mut wallets = Wallets::new().unwrap();
                                    let addr = wallets.create_wallet();
                                    println!("{}'s address is {}", name, addr);
                                    addr
                                });
                            },
                            // 获取当前节点的地址
                            Commands::GetAddress(name) => {
                                println!("{}'s address is {}", name, WALLET_MAP.clone().lock().await.get(&name).unwrap());
                            },
                            Commands::GetBalance(address) => {
                                let wallets = Wallets::new().unwrap();
                                let pub_key_hash = wallets.get_wallet(address.as_str()).unwrap().get_public_key();

                                let balance = self.utxos.get_balance(pub_key_hash);
                                println!("Balance of {}: {}", address, balance);
                            },
                            // 列出本地钱包中的所有地址
                            Commands::ListAddresses => {
                                let wallets = Wallets::new().unwrap();
                                for address in wallets.get_addresses() {
                                    println!("{}", address)
                                }
                            },
                            // 转账交易
                            Commands::Trans{from, to, amount} => {
                                self.transfer(&from, &to, amount.parse::<i32>().unwrap()).await?;
                            },
                        },
                        Err(e) => {
                            error!("Parse command error: {}", e);
                            continue;
                        },
                    }
                },
                // messages是通过P2P网络收到的消息
                messages = self.msg_receiver.recv() => {
                    if let Some(msg) = messages {
                        // Messages是一个enum类型
                        match msg {
                            // 收到其他节点广播的区块链版本（高度）消息
                            Messages::Version{best_height, from_addr} => {
                                self.process_version_msg(best_height, from_addr).await?;
                            },
                            // 收到其他节点广播的区块链完整信息
                            Messages::Blocks{blocks, to_addr, height} => {
                                self.process_blocks_msg(blocks, to_addr, height).await?;
                            },
                            // 收到其他节点广播的挖到的新块
                            Messages::Block{block} => {
                                self.process_block_msg(block).await?;
                            }
                        }
                    }
                },
                event = self.swarm.select_next_some() => { 
                    if let SwarmEvent::NewListenAddr { address, .. } = event { 
                        println!("Listening on {:?}", address); 
                    }
                }
            }
        }
    }
}
