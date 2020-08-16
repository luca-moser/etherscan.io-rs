use std::env::VarError;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::export::Formatter;

use format::*;
use models::*;

mod models;
mod format;

type AsyncError = Box<dyn std::error::Error + Send + Sync>;

const BASE_URL: &str = "https://api.etherscan.io/api";
const ETHERSCANIO_API_TOKEN: &str = "ETHERSCANIO_API_TOKEN";

#[derive(Serialize, Deserialize, Debug)]
struct Response<T>
    where
        T: Debug + Send + Sync
{
    #[serde(deserialize_with = "from_str")]
    status: StatusCode,
    message: String,
    result: T,
}

impl<T: 'static> Response<T> where T: Debug + Send + Sync {
    fn result_or_error(self) -> Result<T, AsyncError> {
        match self.status {
            StatusCode::Error => {
                Err(Box::new(ResponseError { status_code: self.status, message: self.message, result: self.result }))
            }
            _ => Ok(self.result)
        }
    }
}

#[derive(Debug)]
struct ResponseError<R>
    where
        R: Debug + Send + Sync,
{
    status_code: StatusCode,
    message: String,
    result: R,
}

impl<R> fmt::Display for ResponseError<R>
    where
        R: Debug + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "response error with status {}, message: {}, result: {:?}", self.status_code, self.message, self.result)
    }
}

impl<R> std::error::Error for ResponseError<R> where R: Debug + Send + Sync {}

#[derive(Serialize, Deserialize, Debug)]
enum StatusCode {
    Ok,
    Error,
    Unknown,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::Ok => {
                write!(f, "{}", "ok")
            }
            StatusCode::Error => {
                write!(f, "{}", "error")
            }
            StatusCode::Unknown => {
                write!(f, "{}", "unknown")
            }
        }
    }
}

impl FromStr for StatusCode {
    type Err = StatusCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(StatusCode::Ok),
            "0" => Ok(StatusCode::Error),
            _ => Err(StatusCode::Unknown),
        }
    }
}

pub struct API {
    api_token: String,
    client: Client,
}

impl API {
    pub fn new(api_token: &str) -> API {
        API { api_token: api_token.into(), client: reqwest::Client::new() }
    }

    pub fn new_from_env() -> Result<API, VarError> {
        let val = std::env::var(ETHERSCANIO_API_TOKEN)?;
        Ok(API { api_token: val, client: reqwest::Client::new() })
    }

    async fn fetch_balance(&self, uri: String) -> Result<u128, AsyncError> {
        match self.client.get(&uri).send()
            .await?
            .json::<Response<Balance>>()
            .await?
            .result_or_error() {
            Ok(result) => match result.value() {
                Ok(v) => Ok(v),
                Err(e) => Err(Box::new(e)),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn acc_balance(&self, account_addr: &str) -> Result<u128, AsyncError> {
        let uri = format!("{}/?module=account&action=balance&address={}&tag=latest&apikey={}", BASE_URL, account_addr, self.api_token);
        self.fetch_balance(uri).await
    }

    pub async fn estimate_conf_time_for_gas(&self, gas: u128) -> Result<u128, AsyncError> {
        let uri = format!("{}?module=gastracker&action=gasestimate&gasprice={}&apikey={}", BASE_URL, gas, self.api_token);
        self.fetch_balance(uri).await
    }

    pub async fn gas_oracle(&self) -> Result<GasOracle, AsyncError> {
        let uri = format!("{}?module=gastracker&action=gasoracle&apikey={}", BASE_URL, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<GasOracle>>()
            .await?
            .result_or_error()
    }

    pub async fn eth_price(&self) -> Result<ETHPrice, AsyncError> {
        let uri = format!("{}?module=stats&action=ethprice&apikey={}", BASE_URL, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<ETHPrice>>()
            .await?
            .result_or_error()
    }

    pub async fn erc20_token_total_supply(&self, token_contract_addr: &str) -> Result<u128, AsyncError> {
        let uri = format!("{}?module=stats&action=tokensupply&contractaddress={}&apikey={}", BASE_URL, token_contract_addr, self.api_token);
        self.fetch_balance(uri).await
    }

    pub async fn erc20_token_balance_on_account(&self, account_addr: &str, token_contract_addr: &str) -> Result<u128, AsyncError> {
        let uri = format!("{}?module=account&action=tokenbalance&contractaddress={}&address={}&tag=latest&apikey={}", BASE_URL, token_contract_addr, account_addr, self.api_token);
        self.fetch_balance(uri).await
    }

    pub async fn txs_on_account_from_to(&self, account_addr: &str, from_block: u64, end_block: u64) -> Result<Vec<Transaction>, AsyncError> {
        let uri = format!("{}?module=account&action=txlist&address={}{}&sort=asc&apikey={}", BASE_URL, account_addr, parse_block_range(from_block, end_block), self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<Transaction>>>()
            .await?
            .result_or_error()
    }

    pub async fn txs_on_account(&self, account_addr: &str) -> Result<Vec<Transaction>, AsyncError> {
        self.txs_on_account_from_to(account_addr, 0, 0).await
    }

    pub async fn internal_txs_on_account_from_to(&self, account_addr: &str, from_block: u64, end_block: u64) -> Result<Vec<InternalTransaction>, AsyncError> {
        let uri = format!("{}?module=account&action=txlistinternal&address={}{}&sort=asc&apikey={}", BASE_URL, account_addr, parse_block_range(from_block, end_block), self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<InternalTransaction>>>()
            .await?
            .result_or_error()
    }

    pub async fn internal_txs_on_account(&self, addr: &str) -> Result<Vec<InternalTransaction>, AsyncError> {
        self.internal_txs_on_account_from_to(addr, 0, 0).await
    }

    pub async fn internal_txs_from_to(&self, from_block: u64, end_block: u64) -> Result<Vec<InternalTransaction>, AsyncError> {
        let uri = format!("{}?module=account&action=txlistinternal{}&page=1&offset=10&sort=asc&apikey={}", BASE_URL, parse_block_range(from_block, end_block), self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<InternalTransaction>>>()
            .await?
            .result_or_error()
    }

    pub async fn internal_txs_by_tx_hash(&self, tx_hash: &str) -> Result<Vec<InternalTransaction>, AsyncError> {
        let uri = format!("{}?module=account&action=txlistinternal&txhash={}&apikey={}", BASE_URL, tx_hash, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<InternalTransaction>>>()
            .await?
            .result_or_error()
    }

    pub async fn erc20_transfers_on_account_from_to(&self, account_addr: &str, from_block: u64, end_block: u64) -> Result<Vec<ERC20TokenTransferEvent>, AsyncError> {
        let uri = format!("{}?module=account&action=tokentx&address={}{}&sort=asc&apikey={}", BASE_URL, account_addr, parse_block_range(from_block, end_block), self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<ERC20TokenTransferEvent>>>()
            .await?
            .result_or_error()
    }

    pub async fn erc20_transfer_events_on_account(&self, account_addr: &str) -> Result<Vec<ERC20TokenTransferEvent>, AsyncError> {
        self.erc20_transfers_on_account_from_to(account_addr, 0, 0).await
    }

    pub async fn erc20_transfers_on_account_by_contract(&self, account_addr: &str, token_contract_addr: &str) -> Result<Vec<ERC20TokenTransferEvent>, AsyncError> {
        let uri = format!("{}?module=account&action=tokentx&contractaddress={}&address={}&sort=asc&apikey={}", BASE_URL, token_contract_addr, account_addr, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<ERC20TokenTransferEvent>>>()
            .await?
            .result_or_error()
    }

    pub async fn erc271_transfers_on_account_from_to(&self, account_addr: &str, from_block: u64, end_block: u64) -> Result<Vec<ERC721TokenTransferEvent>, AsyncError> {
        let uri = format!("{}?module=account&action=tokennfttx&address={}{}&sort=asc&apikey={}", BASE_URL, account_addr, parse_block_range(from_block, end_block), self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<ERC721TokenTransferEvent>>>()
            .await?
            .result_or_error()
    }

    pub async fn erc271_transfers_on_account(&self, account_addr: &str) -> Result<Vec<ERC721TokenTransferEvent>, AsyncError> {
        self.erc271_transfers_on_account_from_to(account_addr, 0, 0).await
    }

    pub async fn erc271_transfers_on_account_by_contract(&self, account_addr: &str, token_contract_addr: &str) -> Result<Vec<ERC721TokenTransferEvent>, AsyncError> {
        let uri = format!("{}?module=account&action=tokennfttx&contractaddress={}&address={}&sort=asc&apikey={}", BASE_URL, token_contract_addr, account_addr, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<ERC721TokenTransferEvent>>>()
            .await?
            .result_or_error()
    }

    pub async fn mined_blocks_by_account(&self, account_addr: &str) -> Result<Vec<MinedBlock>, AsyncError> {
        let uri = format!("{}?module=account&action=getminedblocks&address={}&blocktype=blocks&apikey={}", BASE_URL, account_addr, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<Vec<MinedBlock>>>()
            .await?
            .result_or_error()
    }

    pub async fn contract_execution_status(&self, tx_hash: &str) -> Result<ContractExecutionStatus, AsyncError> {
        let uri = format!("{}?module=transaction&action=getstatus&txhash={}&apikey={}", BASE_URL, tx_hash, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<ContractExecutionStatus>>()
            .await?
            .result_or_error()
    }

    pub async fn tx_receipt_status(&self, tx_hash: &str) -> Result<TransactionReceiptStatus, AsyncError> {
        let uri = format!("{}?module=transaction&action=gettxreceiptstatus&txhash={}&apikey={}", BASE_URL, tx_hash, self.api_token);
        self.client.get(&uri).send()
            .await?
            .json::<Response<TransactionReceiptStatus>>()
            .await?
            .result_or_error()
    }
}

fn parse_block_range(from: u64, to: u64) -> String {
    if to == 0 {
        return "".to_string();
    }
    format!("&startblock={}&endblock={}", from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAIN_LINK_SMART_CONTRACT_ADDR: &'static str = "0x514910771af9ca656af840dff83e8264ecf986ca";

    fn read_addr_from_env() -> String {
        match std::env::var("TEST_ADDR") {
            Ok(val) => val,
            Err(e) => panic!("couldn't read test addr, did you forget to set the env TEST_ADDR? {}", e)
        }
    }

    #[tokio::test]
    async fn query_balance() {
        let api = API::new_from_env().unwrap();
        match api.acc_balance(&read_addr_from_env()).await {
            Ok(balance) => {
                println!("got balance: {}", balance);
            }
            Err(e) => {
                println!("error occurred while fetching balance: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn query_chainlink_total_supply() {
        let api = API::new_from_env().unwrap();
        match api.erc20_token_total_supply(CHAIN_LINK_SMART_CONTRACT_ADDR).await {
            Ok(supply) => {
                println!("chainlink has a total supply of {}", supply);
            }
            Err(e) => {
                println!("error occurred while fetching chainlink total supply: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn query_gas_oracle() {
        let api = API::new_from_env().unwrap();
        match api.gas_oracle().await {
            Ok(gas_oracle) => {
                println!("gas oracle: {:?}", gas_oracle);
            }
            Err(e) => {
                println!("error occurred while fetching gas oracle: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn query_eth_price() {
        let api = API::new_from_env().unwrap();
        match api.eth_price().await {
            Ok(eth_price) => {
                println!("gas oracle: {:?}", eth_price);
            }
            Err(e) => {
                println!("error occurred while fetching ETH price: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn query_txs() {
        let api = API::new_from_env().unwrap();
        match api.txs_on_account(&read_addr_from_env()).await {
            Ok(txs) => {
                println!("got {} txs", txs.len());
                for tx in txs {
                    println!("{:?}", tx);
                }
            }
            Err(e) => {
                println!("error occurred while fetching txs: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn query_internal_txs() {
        let api = API::new_from_env().unwrap();
        match api.internal_txs_on_account(&read_addr_from_env()).await {
            Ok(txs) => {
                println!("got {} internal txs", txs.len());
                for tx in txs {
                    println!("{:?}", tx);
                }
            }
            Err(e) => {
                println!("error occurred while fetching internal txs: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn erc20_token_transfer_events() {
        let api = API::new_from_env().unwrap();
        match api.erc20_transfer_events_on_account(&read_addr_from_env()).await {
            Ok(erc20_transfer_events) => {
                println!("got {} ERC20 transfer events", erc20_transfer_events.len());
                for erc20_transfer_event in erc20_transfer_events {
                    println!("{:?}", erc20_transfer_event);
                }
            }
            Err(e) => {
                println!("error occurred while fetching ERC20 transfer events: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn erc20_token_transfer_events_by_contract_addr() {
        let api = API::new_from_env().unwrap();
        match api.erc20_transfers_on_account_by_contract(&read_addr_from_env(), CHAIN_LINK_SMART_CONTRACT_ADDR).await {
            Ok(erc20_transfer_events) => {
                println!("got {} ERC20 transfer events", erc20_transfer_events.len());
                for erc20_transfer_event in erc20_transfer_events {
                    println!("{:?}", erc20_transfer_event);
                }
            }
            Err(e) => {
                println!("error occurred while fetching ERC20 transfer events by contract addr: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn erc721_token_transfer_events() {
        let api = API::new_from_env().unwrap();
        match api.erc271_transfers_on_account(&read_addr_from_env()).await {
            Ok(erc721_transfer_events) => {
                println!("got {} ERC721 transfer events", erc721_transfer_events.len());
                for erc721_transfer_event in erc721_transfer_events {
                    println!("{:?}", erc721_transfer_event);
                }
            }
            Err(e) => {
                println!("error occurred while fetching ERC721 transfer events: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn mined_blocks() {
        let api = API::new_from_env().unwrap();
        match api.mined_blocks_by_account(&read_addr_from_env()).await {
            Ok(mined_blocks) => {
                println!("got {} mined blocks", mined_blocks.len());
                for mined_block in mined_blocks {
                    println!("{:?}", mined_block);
                }
            }
            Err(e) => {
                println!("error occurred while fetching mined blocks: {:?}", e);
            }
        }
    }
}
