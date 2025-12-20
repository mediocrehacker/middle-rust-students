use axum::{Router, extract::State, response::Json, routing::get};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

mod pow;

use pow::{Address, Block, Blockchain, Mempool, Miner, Tx};

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

    // В качестве исходных данных создаём три кошелька
    let alice_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let alice_address: Address = alice_signing_key.verifying_key();

    let bob_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let bob_address: Address = bob_signing_key.verifying_key();

    let charlie_signing_key: SigningKey = SigningKey::generate(&mut rng);
    let charlie_address: Address = bob_signing_key.verifying_key();

    // Выполняем несколько транзакций
    let tx1 = Tx::sign(alice_address, bob_address, 100, alice_signing_key);
    let tx2 = Tx::sign(bob_address, charlie_address, 15, charlie_signing_key);

    // Добавляем их в mempool
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

    // Приложение предоставляет REST API,
    // через которое клиенты и майнеры могут взаимодействовать с блокчейном.
    let app = Router::new()
        .route("/", get(index))
        .route("/blocks", get(get_blocks).post(post_blocks))
        .route("/txs", get(get_txs).post(post_txs))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
