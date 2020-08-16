use std::fmt::Debug;
use std::num::ParseIntError;

use serde::{Deserialize, Serialize};

use super::format::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Balance(String);

impl Balance {
    pub fn value(&self) -> Result<u128, ParseIntError> {
        self.0.parse()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    #[serde(deserialize_with = "from_str", rename(deserialize = "blockNumber"))]
    block_number: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "timeStamp"))]
    timestamp: u64,
    hash: String,
    #[serde(deserialize_with = "from_str")]
    nonce: u64,
    #[serde(rename(deserialize = "blockHash"))]
    block_hash: String,
    #[serde(deserialize_with = "from_str", rename(deserialize = "transactionIndex"))]
    transaction_index: u64,
    from: String,
    to: String,
    #[serde(deserialize_with = "from_str")]
    value: i64,
    #[serde(deserialize_with = "from_str")]
    gas: i64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "gasPrice"))]
    gas_price: i64,
    #[serde(rename(deserialize = "isError"))]
    is_error: String,
    #[serde(rename(deserialize = "txreceipt_status"))]
    tx_receipt_status: String,
    input: String,
    #[serde(rename(deserialize = "contractAddress"))]
    contract_address: String,
    #[serde(deserialize_with = "from_str", rename(deserialize = "cumulativeGasUsed"))]
    cumulative_gas_used: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "gasUsed"))]
    gas_used: String,
    #[serde(deserialize_with = "from_str")]
    confirmations: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalTransaction {
    #[serde(deserialize_with = "from_str", rename(deserialize = "blockNumber"))]
    block_number: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "timeStamp"))]
    timestamp: u64,
    hash: String,
    from: String,
    to: String,
    #[serde(deserialize_with = "from_str")]
    value: u128,
    #[serde(rename(deserialize = "contractAddress"))]
    contract_address: String,
    input: String,
    #[serde(rename(deserialize = "type"))]
    tx_type: String,
    #[serde(deserialize_with = "from_str")]
    gas: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "gasUsed"))]
    gas_used: String,
    #[serde(deserialize_with = "from_str", rename(deserialize = "traceId"))]
    trace_id: u64,
    #[serde(rename(deserialize = "isError"))]
    is_error: String,
    #[serde(rename(deserialize = "errCode"))]
    err_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ERC20TokenTransferEvent {
    #[serde(deserialize_with = "from_str", rename(deserialize = "blockNumber"))]
    block_number: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "timeStamp"))]
    timestamp: u64,
    hash: String,
    #[serde(deserialize_with = "from_str")]
    nonce: u64,
    #[serde(rename(deserialize = "blockHash"))]
    block_hash: String,
    from: String,
    #[serde(rename(deserialize = "contractAddress"))]
    contract_address: String,
    to: String,
    #[serde(deserialize_with = "from_str")]
    value: u128,
    #[serde(rename(deserialize = "tokenName"))]
    token_name: String,
    #[serde(rename(deserialize = "tokenSymbol"))]
    token_symbol: String,
    #[serde(deserialize_with = "from_str", rename(deserialize = "tokenDecimal"))]
    token_decimal: u64,
    #[serde(rename(deserialize = "transactionIndex"), deserialize_with = "from_str")]
    transaction_index: u64,
    #[serde(deserialize_with = "from_str")]
    gas: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "gasPrice"))]
    gas_price: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "gasUsed"))]
    gas_used: String,
    #[serde(deserialize_with = "from_str", rename(deserialize = "cumulativeGasUsed"))]
    cumulative_gas_used: u64,
    input: String,
    #[serde(deserialize_with = "from_str")]
    confirmations: u64,
}

pub type ERC721TokenTransferEvent = ERC20TokenTransferEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct MinedBlock {
    #[serde(deserialize_with = "from_str", rename(deserialize = "blockNumber"))]
    block_number: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "timeStamp"))]
    timestamp: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "blockRewards"))]
    block_rewards: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionReceiptStatus {
    #[serde(deserialize_with = "from_str")]
    status: u64,
}

impl TransactionReceiptStatus {
    pub fn status(&self) -> ReceiptStatus {
        match self.status {
            1 => ReceiptStatus::Pass,
            _ => ReceiptStatus::Fail,
        }
    }
}

pub enum ReceiptStatus {
    Pass,
    Fail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContractExecutionStatus {
    #[serde(deserialize_with = "from_str", rename(deserialize = "isError"))]
    is_error: u64,
    #[serde(rename(deserialize = "errDescription"))]
    err_description: String,
}

impl ContractExecutionStatus {
    pub fn status(self) -> ExecutionStatus {
        match self.is_error {
            0 => ExecutionStatus::Pass,
            _ => ExecutionStatus::Error { status_code: self.is_error, description: self.err_description },
        }
    }
}

pub enum ExecutionStatus {
    Pass,
    Error { status_code: u64, description: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GasOracle {
    #[serde(deserialize_with = "from_str", rename(deserialize = "LastBlock"))]
    last_block: u128,
    #[serde(deserialize_with = "from_str", rename(deserialize = "SafeGasPrice"))]
    safe_gas_price: u128,
    #[serde(deserialize_with = "from_str", rename(deserialize = "ProposeGasPrice"))]
    propose_gas_price: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ETHPrice {
    #[serde(deserialize_with = "from_str", rename(deserialize = "ethbtc"))]
    eth_btc: f64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "ethbtc_timestamp"))]
    eth_btc_timestamp: u64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "ethusd"))]
    eth_usd: f64,
    #[serde(deserialize_with = "from_str", rename(deserialize = "ethusd_timestamp"))]
    eth_usd_timestamp: u64,
}