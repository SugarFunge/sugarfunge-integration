use crate::{
    asset::{
        asset_batch_transfer_nft, asset_transfer_nft, get_asset_data, get_web3, AssetBatchTransfer,
        AssetData, AssetTransfer,
    },
    config::Config,
    error::ApiError,
};
use actix_web::{
    post,
    web::{Data, Json},
    Responder,
};
use ethcontract::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, fmt::Debug, str::FromStr};

include!(concat!(env!("OUT_DIR"), "/SugarFungeAsset.rs"));
include!(concat!(env!("OUT_DIR"), "/Wrapped1155Factory.rs"));

#[derive(Serialize, Deserialize, Debug)]
pub struct Wrap1155 {
    pub from: String,
    pub amount: u64,
    pub id: u64,
    pub data: AssetData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchWrap1155 {
    from: String,
    amounts: Vec<u64>,
    ids: Vec<u64>,
    pub data: Vec<AssetData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetWrapped1155 {
    id: u64,
    data: AssetData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Unwrap1155 {
    pub id: u64,
    pub amount: u64,
    pub recipient_address: String,
    pub data: AssetData,
}

pub async fn wrapper_wrap(config: &Config, token: Wrap1155) -> Result<impl Responder, ApiError> {
    let web3 = get_web3(config)?;

    let factory_contract = Wrapped1155Factory::deployed(&web3).await?;

    let transfer = AssetTransfer {
        from: token.from,
        to: format!("0x{:x}", Wrapped1155Factory::address(&factory_contract)),
        amount: token.amount,
        id: token.id,
        data: token.data,
    };

    asset_transfer_nft(config, &transfer).await
}

pub async fn wrapper_batch_wrap(
    config: &Config,
    token: BatchWrap1155,
) -> Result<impl Responder, ApiError> {
    let web3 = get_web3(config)?;

    let factory_contract = Wrapped1155Factory::deployed(&web3).await?;

    let transfer = AssetBatchTransfer {
        from: token.from,
        to: format!("0x{:x}", Wrapped1155Factory::address(&factory_contract)),
        amounts: token.amounts,
        ids: token.ids,
        data: token.data,
    };

    asset_batch_transfer_nft(config, &transfer).await
}

pub async fn wrapper_unwrap(
    config: &Config,
    unwrap: &Unwrap1155,
) -> Result<impl Responder, ApiError> {
    let account = {
        let key: PrivateKey = config.signer_private_key.to_owned();
        Account::Offline(key, Some(config.chain_id))
    };

    let web3 = get_web3(config)?;

    let mut contract = Wrapped1155Factory::deployed(&web3).await?;

    contract.defaults_mut().from = Some(account);

    let sugarfunge_contract = SugarFungeAsset::deployed(&web3).await?;

    let result = contract
        .unwrap(
            H160::from_str(&format!(
                "0x{:x}",
                SugarFungeAsset::address(&sugarfunge_contract)
            ))
            .unwrap(),
            unwrap.id.into(),
            unwrap.amount.into(),
            H160::from_str(&unwrap.recipient_address).unwrap(),
            get_asset_data(
                unwrap.data.name.to_owned(),
                unwrap.data.symbol.to_owned(),
                unwrap.data.decimals,
            ),
        )
        .send()
        .await?;

    Ok(Json(json!({
        "tx": format!("0x{:x}", result.hash()),
        "to": &result.as_receipt().unwrap().to,
        "from": &result.as_receipt().unwrap().from,
        "hash": &result.hash()
    })))
}

pub async fn wrapper_get_wrapped(
    config: &Config,
    wrapped: &GetWrapped1155,
) -> Result<impl Responder, ApiError> {
    let account = {
        let key: PrivateKey = config.signer_private_key.to_owned();
        Account::Offline(key, Some(config.chain_id))
    };

    let web3 = get_web3(config)?;

    let mut contract = Wrapped1155Factory::deployed(&web3).await?;

    contract.defaults_mut().from = Some(account);

    let sugarfunge_contract = SugarFungeAsset::deployed(&web3).await?;

    let result = contract
        .get_wrapped_1155(
            H160::from_str(&format!(
                "0x{:x}",
                SugarFungeAsset::address(&sugarfunge_contract)
            ))
            .unwrap(),
            wrapped.id.into(),
            get_asset_data(
                wrapped.data.name.to_owned(),
                wrapped.data.symbol.to_owned(),
                wrapped.data.decimals,
            ),
        )
        .call()
        .await?;

    Ok(Json(json!({ "tx": format!("0x{:x}", result) })))
}

#[post("wrap_1155")]
async fn wrap_1155(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Wrap1155 = serde_json::from_str(&req_body)?;

    wrapper_wrap(&config, req_data).await
}

#[post("batch_wrap_1155")]
async fn batch_wrap_1155(
    req_body: String,
    config: Data<Config>,
) -> Result<impl Responder, ApiError> {
    let req_data: BatchWrap1155 = serde_json::from_str(&req_body)?;

    wrapper_batch_wrap(&config, req_data).await
}

#[post("unwrap_1155")]
async fn unwrap_1155(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Unwrap1155 = serde_json::from_str(&req_body)?;

    wrapper_unwrap(&config, &req_data).await
}

#[post("get_wrapped_1155")]
async fn get_wrapped_1155(
    req_body: String,
    config: Data<Config>,
) -> Result<impl Responder, ApiError> {
    let req_data: GetWrapped1155 = serde_json::from_str(&req_body)?;

    wrapper_get_wrapped(&config, &req_data).await
}
