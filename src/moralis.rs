use crate::{error::ApiError, config::Config};
use actix_web::{http::StatusCode, post, web::{Data, Json}, Responder};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use log::error;
use snailquote::unescape;

#[derive(Serialize, Deserialize, Debug)]
struct Address {
    address: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Token {
    token_address: String
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountToken {
    address: String,
    token_address: String
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenId {
    token_address: String,
    id: u64
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockNumber {
    block: u64
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct QueryParams {
    chain: String,
    format: String
}

pub async fn moralis_call(config: &Config, url: &String) -> Result<impl Responder, ApiError> {

    let awc_client = awc::Client::new();

    let params = QueryParams {
        chain: "ropsten".to_string(),
        format: "decimal".to_string(), 
    };

    let response = 
        awc_client.get(url)
            .insert_header(("X-API-Key", config.moralis_api_key.to_owned()))
            .query(&params).unwrap()
            .send()
            .await;

    match response {
        Ok(mut response) => {
            let body_str: String = std::str::from_utf8(&response.body().await.unwrap()).unwrap().to_string();
            let body: Json<Value> = Json(serde_json::from_str(&body_str).unwrap());
            
            match response.status() {
                StatusCode::OK => Ok(body),
                _ => {
                    error!("Moralis API request failed: {}", body_str);
                    Err(ApiError::MoralisError)
                }
            }
        },
        Err(_) => Err(ApiError::MoralisError)
    }
}

#[post("get_nfts")]
async fn get_nfts(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Address = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + &unescape(&req_data.address).unwrap() + "/nft";

    moralis_call(&config, &url).await
}

#[post("get_contract_nfts")]
async fn get_contract_nfts(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: AccountToken = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + &unescape(&req_data.address).unwrap() + "/nft/" + &unescape(&req_data.token_address).unwrap();

    moralis_call(&config, &url).await
}

#[post("get_nft_transfers")]
async fn get_nft_transfers(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Address = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.address).unwrap() + "/transfers";

    moralis_call(&config, &url).await
}

#[post("get_nft_transfers_by_block")]
async fn get_nft_transfers_by_block(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: BlockNumber = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "block/" + &req_data.block.to_string() + "/nft/transfers";

    moralis_call(&config, &url).await
}

#[post("get_all_token_ids")]
async fn get_all_token_ids(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Token = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap();

    moralis_call(&config, &url).await
}

#[post("get_contract_nft_transfers")]
async fn get_contract_nft_transfers(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Token = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap() + "/transfers";

    moralis_call(&config, &url).await
}

#[post("get_nft_metadata")]
async fn get_nft_metadata(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Token = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap() + "/metadata";

    moralis_call(&config, &url).await
}

#[post("get_nft_owners")]
async fn get_nft_owners(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: Token = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap() + "/owners";

    moralis_call(&config, &url).await
}

#[post("get_token_id_metadata")]
async fn get_token_id_metadata(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: TokenId = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap() + "/" + &req_data.id.to_string() ;

    moralis_call(&config, &url).await
}

#[post("get_token_id_owners")]
async fn get_token_id_owners(req_body: String, config: Data<Config>) -> Result<impl Responder, ApiError> {
    let req_data: TokenId = match serde_json::from_str(&req_body) {
        Ok(body) => body,
        Err(_) => return Err(ApiError::InvalidRequest)
    };

    let url: String = config.moralis_base_url.to_owned() + "nft/" + &unescape(&req_data.token_address).unwrap() + "/" + &req_data.id.to_string() + "/owners";

    moralis_call(&config, &url).await
}
