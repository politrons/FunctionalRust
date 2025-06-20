// src/main.rs
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// A single block in the chain.
///
/// Fields:
/// * `index`      – position of the block in the chain (genesis = 0)  
/// * `timestamp`  – Unix epoch seconds when the block was mined  
/// * `prev_hash`  – SHA-256 hash of the previous block (or "0" for genesis)  
/// * `data`       – payload; in a real chain this would hold txs or state changes  
/// * `nonce`      – number iterated during Proof-of-Work search  
/// * `hash`       – final SHA-256 digest that satisfies the difficulty target
#[derive(Serialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: i64,
    prev_hash: String,
    data: String,
    nonce: u64,
    hash: String,
}

impl Block {
    /// Mines a new block that satisfies the given `difficulty`.
    ///
    /// The difficulty is expressed as “number of leading zeroes in the hash”.
    /// We loop, bumping `nonce`, until `calc_hash` produces a hash
    /// starting with the required amount of zeroes – classic PoW.
    fn new(index: u64, prev_hash: String, data: String, difficulty: usize) -> Self {
        let mut nonce = 0;
        loop {
            let timestamp = Utc::now().timestamp();
            let hash = Self::calc_hash(index, timestamp, &prev_hash, &data, nonce);

            // Check if the hash matches the target pattern (e.g. "0000…").
            if hash.starts_with(&"0".repeat(difficulty)) {
                return Self {
                    index,
                    timestamp,
                    prev_hash,
                    data,
                    nonce,
                    hash,
                };
            }
            println!("keep grinding");
            nonce += 1; 
        }
    }

    /// Deterministically computes the SHA-256 of the block header fields.
    ///
    /// We concatenate the binary encodings of all header parts, feed them
    /// into the hasher, and output the hex-encoded digest.
    fn calc_hash(
        index: u64,
        timestamp: i64,
        prev_hash: &str,
        data: &str,
        nonce: u64,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(data.as_bytes());
        hasher.update(nonce.to_be_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// An ever-growing vector of blocks plus the consensus parameters.
struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    /// Creates a brand-new chain with a single **genesis block**.
    fn new(difficulty: usize) -> Self {
        let genesis = Block::new(0, "0".into(), "Genesis".into(), difficulty);
        Self {
            blocks: vec![genesis],
            difficulty,
        }
    }

    /// Mines a block with arbitrary `data` and appends it to the chain.
    fn add_block(&mut self, data: String) {
        let prev_hash = self.blocks.last().unwrap().hash.clone();
        let block =
            Block::new(self.blocks.len() as u64, prev_hash, data, self.difficulty);
        self.blocks.push(block);
    }

    /// Re-validates the whole chain from scratch.
    ///
    /// Checks:
    /// 1. `prev_hash` pointer is intact
    /// 2. stored `hash` matches recomputed header hash
    /// 3. hash still satisfies the difficulty target
    fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let curr = &self.blocks[i];
            let prev = &self.blocks[i - 1];

            if curr.prev_hash != prev.hash {
                return false;
            }

            let recalculated = Block::calc_hash(
                curr.index,
                curr.timestamp,
                &curr.prev_hash,
                &curr.data,
                curr.nonce,
            );
            if curr.hash != recalculated {
                return false;
            }

            if !curr
                .hash
                .starts_with(&"0".repeat(self.difficulty))
            {
                return false;
            }
        }
        true
    }
}

fn main() {
    // Create a chain with a tiny PoW target: 4 leading zeros.
    let mut chain = Blockchain::new(4);

    // Add two demo “transactions”.
    chain.add_block("Tx: Pablo → Sky 1 RUSTCOIN".into());
    chain.add_block("Tx: Sky  → Pablo 1 HEART".into());

    // Pretty-print the full chain as JSON and assert validity.
    println!(
        "{}",
        serde_json::to_string_pretty(&chain.blocks).unwrap()
    );
    println!("Chain valid? {}", chain.is_valid());
}
