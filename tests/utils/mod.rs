use actix_http::Request;
use actix_web::{
    self,
    dev::{Service, ServiceResponse},
    test::{self, TestRequest},
    web::Data,
    App, Error,
};
use dotenv::dotenv;
use sugarfunge_integration::{
    asset::{batch_transfer_nft, mint_nft, transfer_nft},
    config::{self, Config},
    moralis::{
        get_all_token_ids, get_contract_nft_transfers, get_contract_nfts, get_nft_metadata,
        get_nft_owners, get_nft_transfers, get_nft_transfers_by_block, get_nfts,
        get_token_id_metadata, get_token_id_owners, get_transaction, Address, QueryParams,
    },
    wrapper::{batch_wrap_1155, get_wrapped_1155, unwrap_1155, wrap_1155},
};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultVec {
    pub amount: String,
    pub token_address: String,
    pub token_hash: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionInfo {
    pub cursor: String,
    pub page: u128,
    pub page_size: u128,
    pub result: Vec<ResultVec>,
    pub status: String,
    pub total: u128,
}

pub fn setup_env_var() -> Config {
    //Setup Env variables
    dotenv().ok();
    config::init()
}

//This function asks moralis for the list of address's nfts and return it parsed
pub async fn get_address_nfts(
    mut app: impl Service<Request, Response = ServiceResponse, Error = Error>,
    mock_address: &String,
) -> TransactionInfo {
    //Mocking request data (MORALIS "get_nft")
    let options = QueryParams {
        chain: Some(String::from("rinkeby")),
        format: Some(String::from("decimal")),
        limit: Some(0),
        offset: Some(0),
    };
    let req_moralis = Address {
        address: mock_address.to_string(),
        options,
    };
    //Fetching NFT amount before minting (MORALIS)
    let moralis_res = TestRequest::post()
        .uri("/get_nfts")
        .set_json(&req_moralis)
        .send_request(&mut app)
        .await;
    assert!(
        moralis_res.status().is_success(),
        "Failed to fetch transaction info"
    );
    let moralis_response: TransactionInfo = test::read_body_json(moralis_res).await;
    moralis_response
}

//This function creates and returns an instance of App
pub async fn get_app(
    env: &Config,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    //Initializing testing app
    test::init_service(
        App::new()
            .service(mint_nft)
            .service(transfer_nft)
            .service(batch_transfer_nft)
            .service(get_nfts)
            .service(get_contract_nfts)
            .service(get_nft_transfers)
            .service(get_transaction)
            .service(get_nft_transfers_by_block)
            .service(get_all_token_ids)
            .service(get_contract_nft_transfers)
            .service(get_nft_metadata)
            .service(get_nft_owners)
            .service(get_token_id_metadata)
            .service(get_token_id_owners)
            .service(wrap_1155)
            .service(batch_wrap_1155)
            .service(unwrap_1155)
            .service(get_wrapped_1155)
            .app_data(Data::new(env.clone())),
    )
    .await
}
//This function return the amount of a nft id from a moralis result vector
pub fn get_nft(moralis_response: TransactionInfo, mock_asset_id: Option<u64>) -> ResultVec {
    //Default value
    let default_nft_vec = ResultVec {
        amount: String::from("0"),
        token_address: String::from(""),
        token_hash: String::from(""),
        token_id: String::from(""),
    };

    if let Some(asset_id) = mock_asset_id {
        moralis_response
            .result
            .iter()
            .find(|element| element.token_id == asset_id.to_string())
            .unwrap_or(&default_nft_vec)
            .clone()
    } else {
        moralis_response
            .result
            .first()
            .unwrap_or(&default_nft_vec)
            .clone()
    }
}

pub fn print_success(text: String) {
    println!("\x1b[0;32m{:#?}\x1b[0m", text);
}
