#![allow(dead_code)]

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};

use super::Tx;

// Мы инкапсулируем реализацию конкретного значения хэша,
// чтобы иметь возможность легко изменить его в будущем
pub type BlockHash = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,                // позицию в цепи
    pub nonce: u64,                // число, найденное майнерами для соблюдения сложности
    pub previous_block: BlockHash, // хэш предыдущего блока (нет у генезис-блока)
    pub hash: BlockHash,           // хэш текущего блока
    pub txs: Vec<Tx>,              // список транзакций с отправителем, получателем и суммой
}

impl Block {
    // При создании нового блока. Хеш-значение рассчитывается автоматически.
    pub fn new(index: u64, nonce: u64, previous_block: BlockHash, txs: Vec<Tx>) -> Self {
        let mut block = Block {
            index,
            nonce,
            previous_block,
            hash: BlockHash::default(),
            txs,
        };
        block.hash = block.calculate_hash();

        block
    }

    // Рассчитывается хеш-значение блока
    pub fn calculate_hash(&self) -> BlockHash {
        let mut hashable_data = self.clone();
        hashable_data.hash = BlockHash::default();
        let serialized = serde_json::to_string(&hashable_data).unwrap();

        let mut block_hash: BlockHash = <[u8; 32]>::default();
        let mut hasher = Sha256::new();

        hasher.input_str(&serialized);
        hasher.result(&mut block_hash);

        block_hash
    }
}
