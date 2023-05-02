use crate::crypto::Hash;
use std::collections::HashMap;
use thiserror::Error;
use ursa::keys::PublicKey;
#[derive(Debug, Clone)]
pub struct Account {
    pub public_key: String,
    pub name: Option<String>,
    pub contact_info: Option<String>,
    pub balance: u32,
}

pub type Accounts = HashMap<u32, Account>;

pub type Assets = HashMap<(u32, String), Asset>;

#[derive(Debug, Clone)]
pub struct Asset {
    pub value: i32,
    pub owner_id: u32,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    CreateAccount {
        public_key: String,
    },
    AddFunds {
        account_id: u32,
        value: i32,
        asset_id: String,
    },
    TransferFunds {
        from_account_id: u32,
        to_account_id: u32,
        value: i32,
        asset_id: String,
    },
    UpdateAccount {
        account_id: u32,
        name: Option<String>,
        contact_info: Option<String>,
    },
    ExecuteSmartContract {
        contract_id: String,
        params: HashMap<String, String>,
        account_id: u32,
    },
    IssueAsset {
        account_id: u32,
        asset_id: String,
        value: i32,
    },
    TransferAsset {
        from_account_id: u32,
        to_account_id: u32,
        asset_id: String,
    },
    RedeemAsset {
        account_id: u32,
        asset_id: String,
        value: i32,
        redeem_in_asset_id: String,
    },
    TransactionCommission {
        account_id: u32,
        value: i32,
    },
}

#[derive(Debug, Clone)]
pub struct Block {
    pub data: Vec<Transaction>,
    pub signature: Vec<u8>,
    pub signer_public_key: PublicKey,
    pub previous_block_hash: Option<Hash>,
}
#[derive(Error, Debug)]

pub enum BlockchainError {
    #[error("account not found")]
    AccountNotFound,

    #[error("asset not found")]
    AssetNotFound,

    #[error("insufficient balance")]
    InsufficientBalance,

    #[error("unknown contract: {0}")]
    UnknownContract(String),

    #[error("parsing parameters for contract failed")]
    FailedToParseParameters,
}

impl Command {
    pub fn execute(
        &self,
        accounts: &mut Accounts,
        assets: &mut Assets,
    ) -> Result<(), BlockchainError> {
        match self {
            Self::CreateAccount { public_key } => {
                accounts.insert(
                    (accounts.len() + 1) as u32,
                    Account {
                        public_key: public_key.clone(),
                        name: None,
                        contact_info: None,
                        balance: 0,
                    },
                );
            }
            Self::AddFunds {
                account_id,
                value,
                asset_id,
            } => {
                assets.insert(
                    (*account_id, asset_id.clone()),
                    Asset {
                        value: *value,
                        owner_id: 0,
                    },
                );
            }
            Self::TransferFunds {
                from_account_id,
                to_account_id,
                value,
                asset_id,
            } => {
                let asset_from = assets
                    .get_mut(&(*from_account_id, asset_id.clone()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                if asset_from.value < *value {
                    return Err(BlockchainError::InsufficientBalance);
                }
                asset_from.value -= *value;

                let asset_to = assets
                    .get_mut(&(*to_account_id, asset_id.clone()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                asset_to.value += *value;
            }
            Self::UpdateAccount {
                account_id,
                name,
                contact_info,
            } => {
                let account = accounts
                    .get_mut(account_id)
                    .ok_or(BlockchainError::AccountNotFound)?;
                if let Some(name) = name {
                    account.name = Some(name.clone());
                }
                if let Some(contact_info) = contact_info {
                    account.contact_info = Some(contact_info.clone());
                }
            }
            Self::ExecuteSmartContract {
                contract_id,
                params,
                account_id,
            } => {
                if contract_id == "transfer_funds" {
                    if let (Some(to_account_id), Some(value), Some(asset_id)) = (
                        params
                            .get("to_account_id")
                            .and_then(|s| s.parse::<u32>().ok()),
                        params.get("value").and_then(|s| s.parse::<i32>().ok()),
                        params.get("asset_id"),
                    ) {
                        Command::TransferFunds {
                            from_account_id: *account_id,
                            to_account_id,
                            value,
                            asset_id: asset_id.clone(),
                        }
                        .execute(accounts, assets)?;
                    } else {
                        return Err(BlockchainError::FailedToParseParameters);
                    }
                } else {
                    return Err(BlockchainError::UnknownContract(contract_id.clone()));
                }
            }
            Self::IssueAsset {
                account_id,
                asset_id,
                value,
            } => {
                let asset = assets
                    .entry((*account_id, asset_id.clone()))
                    .or_insert(Asset {
                        value: 0,
                        owner_id: *account_id,
                    });
                asset.value += *value;
            }
            Self::TransferAsset {
                from_account_id,
                to_account_id,
                asset_id,
            } => {
                let asset = assets
                    .get_mut(&(*from_account_id, asset_id.clone()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                asset.owner_id = *to_account_id;
            }
            Self::RedeemAsset {
                account_id,
                asset_id,
                value,
                redeem_in_asset_id,
            } => {
                let asset = assets
                    .get_mut(&(*account_id, asset_id.clone()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                if asset.value < *value {
                    return Err(BlockchainError::InsufficientBalance);
                }
                asset.value -= *value;

                let redeem_in_asset = assets
                    .get_mut(&(*account_id, redeem_in_asset_id.clone()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                redeem_in_asset.value += *value;
            }
            Self::TransactionCommission { account_id, value } => {
                let asset = assets
                    .get_mut(&(*account_id, "currency".to_string()))
                    .ok_or(BlockchainError::AssetNotFound)?;
                if asset.value < *value {
                    return Err(BlockchainError::InsufficientBalance);
                }
                asset.value -= *value;
            }
        }
        Ok(())
    }
}
