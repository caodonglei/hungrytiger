use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING, KeyPair};
use serde::{Serialize, Deserialize};
use crate::utils::{new_private_key, base58_encode, sha256_digest, ripemd160_digest};

const VERSION: u8 = 0x00;
pub const ADDRESS_CHECKSUM_LEN: usize = 4;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pkcs8: Vec<u8>,         // KeyPair中的私钥
    public_key: Vec<u8>,    // KeyPair中的公钥
}

impl Wallet {
    /*
     * 1. 利用椭圆曲线产生私钥
     * 2. 根据私钥生成秘钥对
     * 3. 从秘钥对导出公钥
     */
    pub fn new() -> Self {
        let pkcs8 = new_private_key();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).unwrap();
        let public_key = key_pair.public_key().as_ref().to_vec();

        Self { 
            pkcs8, 
            public_key 
        }
    }

    // 根据公钥计算地址
    pub fn get_address(&self) -> String {
        // 对公钥计算hash
        let pub_key_hash = hash_pub_key(self.public_key.as_slice());
        let mut payload = vec![];
        // 给哈希值加上版本前缀，这里硬编码为 const VERSION: u8 = 0x00。
        payload.push(VERSION);
        payload.extend(pub_key_hash.as_slice());
        // 计算校验和
        let checksum = checksum(payload.as_slice());
        payload.extend(checksum.as_slice());
        // 使用 Base58 对 version+PubKeyHash+checksum 组合进行编码
        base58_encode(payload.as_slice())
    }

    pub fn get_private_key(&self) -> &[u8] {
        self.pkcs8.as_slice()
    }

    pub fn get_public_key(&self) -> &[u8] {
        self.public_key.as_slice()
    }
}

/*
 * 1. 使用SHA256对公钥进行一次哈希
 * 2. 对结果使用RIPEMD160进行二次哈希
 */
pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = sha256_digest(pub_key);
    let pub_key_ripemd160 = ripemd160_digest(&pub_key_sha256);
    pub_key_ripemd160
}

// 使用 SHA256进行两次哈希。取结果的前四个字节作为校验和。
pub fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha = sha256_digest(payload);
    let second_sha = sha256_digest(&first_sha);
    second_sha[0..ADDRESS_CHECKSUM_LEN].to_vec()
}
