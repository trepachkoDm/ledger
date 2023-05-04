use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use ursa::{
    keys::{PrivateKey, PublicKey},
    signatures::{prelude::Ed25519Sha512, SignatureScheme},
};

use crate::comands::{Block, Command, Transaction};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to sign transaction: {0}")]
    SignTransaction(String),

    #[error("Failed to send transaction: {0}")]
    SendTransaction(String),

    #[error("Failed to create account: {0}")]
    CreateAccount(String),

    #[error("Failed to transfer funds: {0}")]
    TransferFunds(String),

    #[error("Failed to update account: {0}")]
    UpdateAccount(String),

    #[error("Failed to execute smart contract: {0}")]
    ExecuteSmartContract(String),

    #[error("Failed to release asset: {0}")]
    ReleaseAsset(String),

    #[error("Failed to transfer asset: {0}")]
    TransferAsset(String),

    #[error("Failed to redeem asset: {0}")]
    RedeemAsset(String),

    #[error("add funds: {0}")]
    AddFunds(String),

    #[error("issue asset: {0}")]
    IssueAsset(String),

    #[error("transaction commission: {0}")]
    TransactionCommission(String),
}
#[derive(Debug)]
pub struct Client {
    tx: Sender<Block>,
    public_key: PublicKey,
    private_key: PrivateKey,
    peer_rx: Receiver<String>,
}

impl Client {
    pub fn new(
        tx: Sender<Block>,
        public_key: PublicKey,
        private_key: PrivateKey,
        peer_rx: Receiver<String>,
    ) -> Self {
        Self {
            tx,
            public_key,
            private_key,
            peer_rx,
        }
    }
    // метод для получения обновлений от Peer
    pub fn receive_updates(&self) {
        while let Ok(message) = self.peer_rx.recv() {
            println!("Received update: {}", message);
        }
    }

    // функция send_transaction, отвечает за создание блока из транзакции, подписание его закрытым ключом и отправку в канал.
    fn send_transaction(&self, transaction: Transaction) -> Result<(), ClientError> {
        let data = vec![transaction];
        let signature = Ed25519Sha512::new()
            .sign(format!("{:?}", &data).as_bytes(), &self.private_key)
            .map_err(|e| ClientError::SignTransaction(e.to_string()))?;
        self.tx
            .send(Block {
                signature,
                data,
                signer_public_key: self.public_key.clone(),
                previous_block_hash: None,
            })
            .map_err(|e| ClientError::SendTransaction(e.to_string()))?;
        Ok(())
    }

    pub fn create_account(&self) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::CreateAccount {
                public_key: self.public_key.to_string(),
            },
        })
        .map_err(|_| ClientError::CreateAccount("create account".to_string()))
    }

    pub fn transfer_funds(
        &self,
        from_account_id: u32,
        to_account_id: u32,
        value: i32,
        asset_id: String,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::TransferFunds {
                from_account_id,
                to_account_id,
                value,
                asset_id,
            },
        })
        .map_err(|_| ClientError::TransferFunds("transfer funds".to_string()))
    }

    pub fn update_account(
        &self,
        account_id: u32,
        name: Option<String>,
        contact_info: Option<String>,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::UpdateAccount {
                account_id,
                name,
                contact_info,
            },
        })
        .map_err(|_| ClientError::UpdateAccount("update account".to_string()))
    }
    pub fn execute_smart_contract(
        &self,
        account_id: u32,
        contract_id: String,
        params: HashMap<String, String>,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::ExecuteSmartContract {
                account_id,
                contract_id,
                params,
            },
        })
        .map_err(|_| ClientError::ExecuteSmartContract("execute smart contract".to_string()))
    }

    pub fn release_asset(
        &self,
        account_id: u32,
        asset_id: String,
        value: i32,
        redeem_in_asset_id: String,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::RedeemAsset {
                account_id,
                asset_id,
                value,
                redeem_in_asset_id,
            },
        })
        .map_err(|_| ClientError::ReleaseAsset("release asset".to_string()))
    }

    pub fn transfer_asset(
        &self,
        from_account_id: u32,
        to_account_id: u32,
        asset_id: String,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::TransferAsset {
                from_account_id,
                to_account_id,
                asset_id,
            },
        })
        .map_err(|_| ClientError::TransferAsset("transfer asset".to_string()))
    }

    pub fn redeem_asset(
        &self,
        account_id: u32,
        asset_id: String,
        value: i32,
        redeem_in_asset_id: String,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::RedeemAsset {
                account_id,
                asset_id,
                value,
                redeem_in_asset_id,
            },
        })
        .map_err(|_| ClientError::RedeemAsset("redeem asset".to_string()))
    }

    pub fn add_funds(
        &self,
        account_id: u32,
        value: i32,
        asset_id: String,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::AddFunds {
                account_id,
                value,
                asset_id,
            },
        })
        .map_err(|_| ClientError::AddFunds("add funds".to_string()))
    }

    pub fn issue_asset(
        &self,
        account_id: u32,
        asset_id: String,
        value: i32,
    ) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::IssueAsset {
                account_id,
                asset_id,
                value,
            },
        })
        .map_err(|_| ClientError::IssueAsset("issue asset".to_string()))
    }

    pub fn transaction_commission(&self, account_id: u32, value: i32) -> Result<(), ClientError> {
        self.send_transaction(Transaction {
            command: Command::TransactionCommission { account_id, value },
        })
        .map_err(|_| ClientError::TransactionCommission("transaction commission".to_string()))
    }
}
