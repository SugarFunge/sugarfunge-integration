use crate::{error::ApiError, config::Config};
use std::{env, str::FromStr};
use ethcontract::{prelude::*, web3::ethabi::{Token, encode}};
use actix_web::{post, web::{Data, Json}, Responder};
use serde::{Serialize, Deserialize};
use serde_json::json;
use rustc_hex::ToHex;

include!(concat!(env!("OUT_DIR"), "/SugarFungeAsset.rs"));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetData {
    pub name: String,
    pub symbol: String,
    pub decimals: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetMint {
    account: String,
    amount: u64,
    id: u64,
    data: AssetData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetTransfer {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub id: u64,
    pub data: AssetData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetBatchTransfer {
    pub from: String,
    pub to: String,
    pub amounts: Vec<u64>,
    pub ids: Vec<u64>,
    pub data: Vec<AssetData>,
}

pub fn get_web3(config: &Config) -> Result<Web3<Http>, ApiError> {

    let infura_url = {
        format!("https://ropsten.infura.io/v3/{}", config.project_id)
    };

    match Http::new(&infura_url) {
        Ok(http) => Ok(Web3::new(http)),
        Err(_) => Err(ApiError::TransportError),
    }
}

pub fn calculate_bytes_32(value: String) -> String {

    let string_as_hex: String = value.as_bytes().to_hex();
    let string_length = string_as_hex.len();
    let zero_padding = 62 - string_length;
    let string_zero_padding = "0".repeat(zero_padding);
    let string_length_as_hex = format!("{:x}", string_length);

    format!("{}{}{}", string_as_hex, string_zero_padding, string_length_as_hex)
}

pub fn get_asset_data(name: String, symbol: String, decimals: u64) -> Bytes<Vec<u8>> {

    ethcontract::Bytes(encode(
        &[
            Token::Tuple(
            [
                Token::String(name), 
                Token::String(symbol), 
                Token::Uint(decimals.into())
            ]
            .to_vec())
        ])
    )
}

pub fn get_batch_asset_data(data: Vec<AssetData>) -> Bytes<Vec<u8>> {

    let mut batch_vec: Vec<String> = [].to_vec();

    for asset in data {
        batch_vec.push(
            format!("{}{}{:x}", 
                calculate_bytes_32(asset.name), 
                calculate_bytes_32(asset.symbol), 
                asset.decimals
            )
        );
    }

    let batch_string = format!("0x{}", batch_vec.join(""));
    let batch_data = batch_string.into_bytes();

    ethcontract::Bytes(batch_data)
}

pub async fn asset_mint_nft(config: &Config, mint: &AssetMint) -> Result<impl Responder, ApiError> {

    let account = {
        let key: PrivateKey = config.private_key.to_owned();
        Account::Offline(key, Some(config.chain_id))
    };

    let web3 = get_web3(config)?;

    let mut contract =  SugarFungeAsset::deployed(&web3).await?;

    contract.defaults_mut().from = Some(account);

    let result = contract.mint(
        H160::from_str(&mint.account).unwrap(), 
        mint.id.into(), 
        mint.amount.into(), 
        get_asset_data(mint.data.name.to_owned(), mint.data.symbol.to_owned(), mint.data.decimals))
        .send()
        .await?;

    Ok(Json(json!({
        "tx": format!("0x{:x}", result.hash())
    })))
}

pub async fn asset_transfer_nft(config: &Config, transfer: &AssetTransfer) -> Result<impl Responder, ApiError> {

    let account = {
        let key: PrivateKey = config.private_key.to_owned();
        Account::Offline(key, Some(config.chain_id))
    };

    let web3 = get_web3(config)?;

    let mut contract = SugarFungeAsset::deployed(&web3).await?;

    contract.defaults_mut().from = Some(account);

    let result = contract.safe_transfer_from (
        H160::from_str(&transfer.from).unwrap(), 
        H160::from_str(&transfer.to).unwrap(), 
        transfer.id.into(), 
        transfer.amount.into(),
        get_asset_data(transfer.data.name.to_owned(), transfer.data.symbol.to_owned(), transfer.data.decimals))
        .send()
        .await?;

    Ok(Json(json!({
        "tx": format!("0x{:x}", result.hash())
    })))
}

pub async fn asset_batch_transfer_nft(config: &Config, transfer: &AssetBatchTransfer) -> Result<impl Responder, ApiError> {

    let account = {
        let key: PrivateKey = config.private_key.to_owned();
        Account::Offline(key, Some(config.chain_id))
    };

    let web3 = get_web3(config)?;

    let mut contract = SugarFungeAsset::deployed(&web3).await?;

    contract.defaults_mut().from = Some(account);

    let result = contract.safe_batch_transfer_from (
        H160::from_str(&transfer.from).unwrap(), 
        H160::from_str(&transfer.to).unwrap(), 
        transfer.ids.iter().map(|x| x.to_owned().into()).collect(), 
        transfer.amounts.iter().map(|x| x.to_owned().into()).collect(),
        get_batch_asset_data(transfer.data.to_vec()))
        .send()
        .await?;

    Ok(Json(json!({
        "tx": format!("0x{:x}", result.hash())
    })))
}

#[post("mint_nft")]
async fn mint_nft(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: AssetMint = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    asset_mint_nft(&config, &req_data).await
}

#[post("transfer_nft")]
async fn transfer_nft(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: AssetTransfer = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    asset_transfer_nft(&config, &req_data).await
}

#[post("batch_transfer_nft")]
async fn batch_transfer_nft(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: AssetBatchTransfer = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    asset_batch_transfer_nft(&config, &req_data).await
}
