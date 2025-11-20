use anyhow::Result;
use axum::{extract::State, response::Json, routing::get, Router};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use ed25519_dalek::ed25519::signature::Signer;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

type Address = VerifyingKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxUnsigned {
    sender: Address,
    recipient: Address,
    amount: u64,
}

impl TxUnsigned {
    fn new(sender: Address, recipient: Address, amount: u64) -> Self {
        TxUnsigned {
            sender,
            recipient,
            amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tx {
    sender: Address,
    recipient: Address,
    amount: u64,
    signature: Signature,
}

impl Tx {
    fn sign(sender: Address, recipient: Address, amount: u64, signinig_key: SigningKey) -> Tx {
        let unsigned = TxUnsigned::new(sender, recipient, amount);
        let serialized = serde_json::to_string(&unsigned).unwrap();
        let signature: Signature = signinig_key.sign(&serialized.into_bytes());

        Tx {
            sender,
            recipient,
            amount,
            signature,
        }
    }

    fn is_valid(&self) -> bool {
        let verifying_key: VerifyingKey = self.sender;
        let unsigned = TxUnsigned::new(self.sender, self.recipient, self.amount);
        let serialized = serde_json::to_string(&unsigned).unwrap();

        verifying_key
            .verify(&serialized.into_bytes(), &self.signature)
            .is_ok()
    }
}

type BlockHash = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    nonce: u64,
    previous_block: BlockHash,
    hash: BlockHash,
    txs: Vec<Tx>,
}

impl Block {
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

#[derive(Error, PartialEq, Debug)]
enum BlockchainError {
    #[error("Invalid previous_hash")]
    PreviousHashMismatch,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid difficulty")]
    IncorrectDifficulty,
}

const DIFFICULTY: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    fn new() -> Blockchain {
        let genesis_block = Blockchain::create_genesis_block();
        let blocks = vec![genesis_block];

        Blockchain { blocks }
    }

    fn create_genesis_block() -> Block {
        let index = 0;
        let nonce = 0;
        let previous_hash = BlockHash::default();
        let transactions = Vec::new();

        Block::new(index, nonce, previous_hash, transactions)
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let blocks = &self.blocks;
        let last = &self.blocks[blocks.len() - 1];
        if block.previous_block != last.hash {
            return Err(BlockchainError::PreviousHashMismatch.into());
        }

        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidHash.into());
        }

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

fn leading_zeros(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .take_while(|&&b| b == 0)
        .count()
        .try_into()
        .unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mempool {
    transactions: Vec<Tx>,
}

impl Mempool {
    fn new() -> Self {
        Mempool {
            transactions: vec![],
        }
    }

    fn push(&mut self, tx: Tx) {
        if tx.is_valid() {
            self.transactions.push(tx);
        }
    }

    fn get_txs(&self) -> Vec<Tx> {
        self.transactions.clone()
    }
}

pub struct Miner {
    max_nonce: u64,
}

impl Miner {
    fn new(max_nonce: u64) -> Miner {
        Miner { max_nonce }
    }

    fn mine_block(&self, last_block: &Block, transactions: &[Tx]) -> Option<Block> {
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

struct AppState {
    blockchain: Mutex<Blockchain>,
    pool: Mutex<Mempool>,
}

async fn index() -> &'static str {
    "Blockchain is live!"
}

async fn get_txs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let pool = state.pool.lock().await;
    let txs = pool.get_txs();
    let serialized = serde_json::to_value(txs).unwrap();
    Json(serialized)
}

async fn post_txs(State(state): State<Arc<AppState>>, tx_json: Json<Tx>) -> Json<Value> {
    let tx = tx_json.0;
    let mut pool = state.pool.lock().await;
    pool.push(tx);

    let txs = pool.get_txs();
    let serialized = serde_json::to_value(txs).unwrap();
    Json(serialized)
}

async fn get_blocks(State(state): State<Arc<AppState>>) -> Json<Value> {
    let blockchain = &state.blockchain.lock().await;
    let blocks = blockchain.get_blocks();
    let serialized = serde_json::to_value(blocks).unwrap();
    Json(serialized)
}

async fn post_blocks(State(state): State<Arc<AppState>>, block_json: Json<Block>) -> Json<Value> {
    let block = block_json.0;
    let mut blockchain = state.blockchain.lock().await;
    let _ = blockchain.add_block(block);

    let blocks = blockchain.get_blocks();
    let serialized = serde_json::to_value(blocks).unwrap();
    Json(serialized)
}

#[tokio::main]
async fn main() {
    let mut rng = OsRng;

    let alice_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let alice_address: Address = alice_signing_key.verifying_key();

    let bob_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let bob_address: Address = bob_signing_key.verifying_key();

    let charlie_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let charlie_address: Address = bob_signing_key.verifying_key();

    let tx1 = Tx::sign(alice_address, bob_address, 100, alice_signing_key);
    let tx2 = Tx::sign(bob_address, charlie_address, 15, charlie_signing_key);

    let miner = Miner::new(1_000_000);
    let mut blockchain = Blockchain::new();
    let mut mempool = Mempool::new();
    mempool.push(tx1);
    mempool.push(tx2);
    let block = miner
        .mine_block(blockchain.blocks.last().unwrap(), &mempool.transactions)
        .unwrap();
    let _ = blockchain.add_block(block);

    // println!("Blockchain state: {:?}", blockchain);

    let shared_state = Arc::new(AppState {
        blockchain: Mutex::new(blockchain),
        pool: Mutex::new(mempool),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/blocks", get(get_blocks).post(post_blocks))
        .route("/txs", get(get_txs).post(post_txs))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
