use anyhow::Result;
use serde::{Serialize, Deserialize};
use crypto::{sha3::Sha3, digest::Digest};

use crate::error::BlockchainError;

/* 区块体数据序列化 */
pub fn serialize<T>(data: &T) -> Result<Vec<u8>, BlockchainError>
where
    T: Serialize + ?Sized
{
    Ok(bincode::serialize(data)?)
}

/* 区块体数据反序列化 */
#[allow(dead_code)]
pub fn deserialize<'a, T>(data: &'a [u8]) -> Result<T, BlockchainError>
where
    T: Deserialize<'a> + ?Sized
{
    Ok(bincode::deserialize(data)?)
}

/* 根据字节流计算hash值，用于计算一个区块的hash */
pub fn hash_to_str(data: &[u8]) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input(data);
    hasher.result_str()
}

/* 根据字节流计算hash值，用于计算一个区块的hash */
#[allow(dead_code)]
pub fn hash_to_u8(data: &[u8], out: &mut [u8]) {
    let mut hasher = Sha3::sha3_256();
    hasher.input(data);
    hasher.result(out)
}

