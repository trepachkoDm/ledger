use crate::comands::{Block, Command, Transaction};
use crate::crypto::calculate_random_number;
use crate::storage::Storage;
use crate::{crypto, Hash};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use ursa::signatures::SignatureScheme;
use ursa::{keys::PublicKey, signatures::prelude::Ed25519Sha512};
#[derive(Debug, Error)]
pub enum StartError {
    #[error("Failed to start: {0}")]
    ErrStart(String),
}

#[derive(Debug)]
pub struct Peer {
    pub id: u8,
    pub rx: Receiver<Block>,
    pub txs: Vec<Sender<Block>>,
    pub storage: Storage,
    pub trusted_public_keys: Vec<PublicKey>,
    pub stake: u32,
}

impl Peer {
    pub fn new(
        id: u8,
        rx: Receiver<Block>,
        txs: Vec<Sender<Block>>,
        stake: u32,
        trusted_public_keys: Vec<PublicKey>,
    ) -> Self {
        Self {
            id,
            rx,
            txs,
            trusted_public_keys,
            stake,
            storage: Storage::new(),
        }
    }

    pub fn start(mut self) -> Result<(), StartError> {
        const TOTAL_STAKE: u32 = 100;
        for i in 0..3 {
            println!("ROUND: {} ____________", i);
            println!("{:?}", &self);
            thread::sleep(Duration::from_millis(10000));
            let prev_block_hash = match self.storage.blockchain.last() {
                Some(block) => crypto::hash(block),
                None => vec![],
            };
            if self.should_propose_block(TOTAL_STAKE, prev_block_hash) {
                let block = self.create_block();
                match self.storage.add_block(block.clone()) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(StartError::ErrStart(format!(
                            "Failed to add block: {}",
                            err
                        )))
                    }
                }

                for tx in &self.txs {
                    tx.send(block.clone()).unwrap();
                }
            }
            while let Ok(block) = self.rx.try_recv() {
                if self.is_valid_block(&block) {
                    match self.storage.add_block(block.clone()) {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(StartError::ErrStart(format!(
                                "Failed to add block: {}",
                                err
                            )))
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_block(&self) -> Block {
        let data = vec![Transaction {
            command: Command::CreateAccount {
                public_key: format!("peer{}_key", self.id),
            },
        }];
        let (public_key, private_key) = Ed25519Sha512::new().keypair(None).unwrap();
        Block {
            signature: Ed25519Sha512::new()
                .sign(format!("{:?}", &data).as_bytes(), &private_key)
                .unwrap(),
            data,
            signer_public_key: public_key,
            previous_block_hash: self.storage.blockchain.last().map(crypto::hash),
        }
    }

    pub fn is_valid_block(&self, block: &Block) -> bool {
        let verified = Ed25519Sha512::new()
            .verify(
                format!("{:?}", &block.data).as_bytes(),
                &block.signature,
                &block.signer_public_key,
            )
            .is_ok();
        let is_trusted = self
            .trusted_public_keys
            .iter()
            .any(|key| key == &block.signer_public_key);

        verified && is_trusted
    }

    pub fn should_propose_block(&self, total_stake: u32, prev_block_hash: Hash) -> bool {
        let random_number = calculate_random_number(prev_block_hash);
        self.stake as f64 / total_stake as f64 > random_number
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use ursa::signatures::{prelude::Ed25519Sha512, SignatureScheme};

    use crate::{
        comands::{Block, Command, Transaction},
        peer::Peer,
    };

    #[test]
    fn test_create_peer() {
        let (tx, rx) = channel();
        let peer = Peer::new(1, rx, vec![tx], 50, vec![]);
        assert_eq!(peer.id, 1);
        assert_eq!(peer.txs.len(), 1);
        assert_eq!(peer.stake, 50);
        assert_eq!(peer.trusted_public_keys.len(), 0);
        assert!(peer.storage.blockchain.is_empty());
    }

    #[test]
    fn test_create_block() {
        let (tx, rx) = channel();
        let peer = Peer::new(1, rx, vec![tx], 50, vec![]);
        let block = peer.create_block();
        assert_eq!(block.data.len(), 1);
        assert!(!block.signature.is_empty());
        assert!(!block.signer_public_key.is_empty());
    }

    #[test]
    fn test_should_propose_block() {
        let (tx, rx) = channel();
        let peer = Peer::new(1, rx, vec![tx], 50, vec![]);
        assert!(peer.should_propose_block(100, vec![]));
    }
    #[test]
    fn test_is_valid_block() {
        let (tx, rx) = channel();
        let (public_key, private_key) = Ed25519Sha512::new().keypair(None).unwrap();
        let public_key_clone = public_key.clone();
        let block = Block {
            signature: Ed25519Sha512::new()
                .sign(
                    format!(
                        "{:?}",
                        &vec![Transaction {
                            command: Command::CreateAccount {
                                public_key: String::from("test"),
                            },
                        }]
                    )
                    .as_bytes(),
                    &private_key,
                )
                .unwrap(),
            data: vec![Transaction {
                command: Command::CreateAccount {
                    public_key: String::from("test"),
                },
            }],
            signer_public_key: public_key,
            previous_block_hash: None,
        };
        let peer = Peer::new(1, rx, vec![tx], 50, vec![public_key_clone]);
        assert!(peer.is_valid_block(&block));
    }
}
