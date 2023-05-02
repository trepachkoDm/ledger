use crate::{
    comands::{Accounts, Assets, Block},
    crypto,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Failed to execute command: {0}")]
    CommandExecutionError(String),
}

#[derive(Debug, Clone)]
pub struct Storage {
    pub blockchain: Vec<Block>,
    pub accounts: Accounts,
    pub assets: Assets,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            blockchain: Vec::new(),
            accounts: HashMap::new(),
            assets: HashMap::new(),
        }
    }

    pub fn add_block(&mut self, mut block: Block) -> Result<(), StorageError> {
        for command in block.data.iter().map(|transaction| &transaction.command) {
            match command.execute(&mut self.accounts, &mut self.assets) {
                Ok(_) => (),
                Err(e) => return Err(StorageError::CommandExecutionError(e.to_string())),
            }
        }
        if let Some(last_block) = self.blockchain.last() {
            block.previous_block_hash = Some(crypto::hash(last_block));
        }
        self.blockchain.push(block);
        Ok(())
    }
}
