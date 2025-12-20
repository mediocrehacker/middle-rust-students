#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Block, BlockHash};

// Типы ошибок, возвращаемые при попытке добавить блоки с недопустимыми полями
#[derive(Error, PartialEq, Debug)]
enum BlockchainError {
    #[error("Invalid previous_hash")]
    PreviousHashMismatch,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid difficulty")]
    IncorrectDifficulty,
}

// Урвень сложности блокчейн системы
pub const DIFFICULTY: u32 = 2;

// Структура блокчейн содержит все существующие блоки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        // При первом создании блокчейн, добавляется блок генезиса
        let genesis_block = Blockchain::create_genesis_block();
        let blocks = vec![genesis_block];

        Blockchain { blocks }
    }

    pub fn create_genesis_block() -> Block {
        let index = 0;
        let nonce = 0;
        let previous_hash = BlockHash::default();
        let transactions = Vec::new();

        Block::new(index, nonce, previous_hash, transactions)
    }

    // Пытается добавить новый блок в блокчейн
    // Проверяется соответствие значений нового блока состоянию блокчейна
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let blocks = &self.blocks;
        let last = &self.blocks[blocks.len() - 1];
        // Проверка корректности указателя на предидущий блок
        if block.previous_block != last.hash {
            return Err(BlockchainError::PreviousHashMismatch.into());
        }

        // Проверка соответствия хеша транзакций
        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidHash.into());
        }

        // Проверка уровня сложности
        if leading_zeros(&block.hash) < DIFFICULTY {
            return Err(BlockchainError::IncorrectDifficulty.into());
        }

        self.blocks.push(block);

        Ok(())
    }

    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }
}

// Вспомогательная функция для проверки уровня сложности
pub fn leading_zeros(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .take_while(|&&b| b == 0)
        .count()
        .try_into()
        .unwrap()
}
