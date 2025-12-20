#![allow(dead_code)]

use super::{Block, DIFFICULTY, Tx, leading_zeros};

// Майнер
pub struct Miner {
    max_nonce: u64,
}

impl Miner {
    // При создании Майнера необходимо указать максимальное значение nonce
    pub fn new(max_nonce: u64) -> Miner {
        Miner { max_nonce }
    }

    // Пытается найти следующий валидный блок блокчейна
    // Он создаёт блоки с разными значениями nonce, пока не найдётся хеш, соответствующий сложности
    // Возвращает либо валидный блок, либо значение None, если блок не найден
    pub fn mine_block(&self, last_block: &Block, transactions: &[Tx]) -> Option<Block> {
        for nonce in 0..self.max_nonce {
            let next_block = self.create_next_block(last_block, transactions.to_owned(), nonce);
            // println!("mining block {:?}", hex::encode(&next_block.hash));

            if leading_zeros(&next_block.hash) >= DIFFICULTY {
                // println!("Block idex: {}, nonce: {}", &next_block.index, &next_block.nonce);
                return Some(next_block);
            }
        }
        None
    }

    fn create_next_block(&self, last_block: &Block, transactions: Vec<Tx>, nonce: u64) -> Block {
        let index = last_block.index + 1;
        let previous_hash = last_block.hash;

        Block::new(index, nonce, previous_hash, transactions)
    }
}
