mod utils;
use crate::utils::{
    get_address_nfts, get_app, get_nft, print_success, setup_env_var, TransactionInfo,
};
use actix_web::{
    self,
    test::{self, TestRequest},
};
use ethcontract::{
    transaction::confirm::{wait_for_confirmation, ConfirmParams},
    H256,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::println as info;
use sugarfunge_integration::{
    asset::{get_web3, AssetData, AssetMint, AssetTransfer},
    wrapper::{Unwrap1155, Wrap1155},
};

//Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    tx: String,
    to: String,
    from: String,
    hash: H256,
}

//ASSET

//Testing Mint proccess
#[actix_web::test]
async fn test_1_asset_mint() {
    //Setup Env variables
    let env = utils::setup_env_var();

    //Initialize rng to get random mock data
    let mut rng = rand::thread_rng();

    info!("\n\n Testing Mint proccess");

    //Mock Data
    let mock_address: String = env.signer_address.clone();
    let mock_asset_id: u64 = rng.gen_range(0..10);
    let mock_amount: u64 = rng.gen_range(50..100);

    //Constructing the mint request
    let req_data = AssetMint {
        account: mock_address.clone(),
        amount: mock_amount,
        id: mock_asset_id,
        data: AssetData {
            name: String::from("name"),
            symbol: String::from("symbol"),
            decimals: 8,
        },
    };

    //Initializing testing app
    let mut app = get_app(&env).await;

    //Fetching moralis to get address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;

    //Saving amount into a variable
    let amount_before_mint: u64 = get_nft(moralis_response.clone(), Some(mock_asset_id))
        .amount
        .clone()
        .parse::<u64>()
        .unwrap();

    info!(
        "Amount of address asset before minting -> {:?}",
        amount_before_mint
    );

    info!("Attempting to mint {:?} assets", mock_amount);

    //Fetching mint nft (INFURA)
    let res = TestRequest::post()
        .uri("/mint_nft")
        .set_json(&req_data)
        .send_request(&mut app)
        .await;
    assert!(res.status().is_success(), "Failed to mint NFT");
    //Parsing the response
    let response: ApiResponse = test::read_body_json(res).await;

    //Initializing web3
    let web3 = get_web3(&env).unwrap();

    //Waiting at least 10 confirmations to fetch the data again
    wait_for_confirmation(&web3, response.hash, ConfirmParams::with_confirmations(10))
        .await
        .expect("Failed while waiting for blockchain confirmation");

    //RE-Fetching address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;

    //Extracting amount from a specific token id
    let amount_after_mint: u64 = get_nft(moralis_response.clone(), Some(mock_asset_id))
        .amount
        .clone()
        .parse::<u64>()
        .unwrap();
    info!(
        "Amount of address's asset after minting -> {:?}",
        amount_after_mint
    );

    //Checking if the from address is the same as the transaction from address
    assert_eq!(&response.from.to_lowercase(), &mock_address.to_lowercase());
    //Checking if the amount that user's hold is what it has plus the minted amount
    let expected_token_amount = amount_before_mint + mock_amount;
    assert_eq!(expected_token_amount, amount_after_mint);

    //Print success message on terminal
    print_success("Mint NFT Test PASSED".to_string());
}
//Testing Transfer proccess
#[actix_web::test]
async fn test_2_asset_transfer() {
    //Setup Env variables
    let env = utils::setup_env_var();

    //Initialize rng to get random mock data
    let mut rng = rand::thread_rng();

    //Initializing testing app
    let mut app = get_app(&env).await;

    info!("\n\n Testing Transfer proccess");

    //Mock Data
    let mock_address = &env.signer_address; //From address
    let mock_second_address: String = String::from(&env.testing_address); //To address
    //Fetching moralis to get address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;
    let mock_asset_id: u64 = moralis_response
        .result
        .first()
        .unwrap()
        .token_id
        .parse::<u64>()
        .unwrap();

    //Saving amount into a variable
    let amount_before_transfer: u64 = get_nft(moralis_response.clone(), Some(mock_asset_id))
        .amount
        .clone()
        .parse::<u64>()
        .unwrap();

    //Random amount to transfer (from 1 to 20)
    let mock_amount: u64 = rng.gen_range(1..20);

    //Constructing the transfer request
    let req_data = AssetTransfer {
        from: mock_address.clone(),
        to: mock_second_address.clone(),
        amount: mock_amount.clone(),
        id: mock_asset_id,
        data: AssetData {
            name: String::from("name"),
            symbol: String::from("symbol"),
            decimals: 8,
        },
    };

    info!(
        "Amount of address asset before transfering -> {:?}",
        &amount_before_transfer
    );

    info!("Attempting to transfer {:?} assets", mock_amount);

    //Fetching transfer nft (INFURA)
    let res = TestRequest::post()
        .uri("/transfer_nft")
        .set_json(&req_data)
        .send_request(&mut app)
        .await;
    assert!(res.status().is_success(), "Failed to transfer NFT");
    //Parsing the response
    let response: ApiResponse = test::read_body_json(res).await;

    //Initializing web3
    let web3 = get_web3(&env).unwrap();
    //Waiting at least 10 confirmations to fetch the data again
    wait_for_confirmation(&web3, response.hash, ConfirmParams::with_confirmations(10))
        .await
        .expect("Failed while waiting for blockchain confirmation");

    //Fetching moralis to get address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;

    let amount_after_transfer: u64 = get_nft(moralis_response.clone(), Some(mock_asset_id))
        .amount
        .clone()
        .parse::<u64>()
        .unwrap();
    info!(
        "Amount of address's asset after transfer -> {:?}",
        amount_after_transfer
    );

    //Checking if the from address is the same as the transaction from address
    assert_eq!(&response.from.to_lowercase(), &mock_address.to_lowercase());

    //Checking if the amount that user's hold is what it has minus the transferred amount
    let expected_token_amount = amount_before_transfer - mock_amount;
    assert_eq!(expected_token_amount, amount_after_transfer);
    
    //Print success message on terminal
    print_success("Transfer NFT Test PASSED".to_string());
}

//WRAPPER

//Testing Wrapper proccess
#[actix_web::test]
async fn test_3_asset_wrap() {
    //Setup Env variables
    let env = setup_env_var();

    //Initialize rng to get random mock data
    let mut rng = rand::thread_rng();

    //Initializing testing app
    let mut app = get_app(&env).await;

    info!("\n\n Testing Wrapper proccess");

    //Mock Data
    let mock_address: String = env.signer_address.clone();

    //Fetching moralis to get address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;
    let returned_nft = get_nft(moralis_response.clone(), None);
    let mock_asset_id: u64 = returned_nft.token_id.parse::<u64>().unwrap();
    
    //Saving amount into a variable
    let amount_before_wrap: u64 = returned_nft.amount.clone().parse::<u64>().unwrap();
    if amount_before_wrap == 0 {
        panic!("Address doesn't have a nft");
    }
    let mock_amount: u64 = rng.gen_range(5..20);

    //Mocking request data (INFURA)
    //Constructing the transfer request
    let req_data = Wrap1155 {
        from: mock_address.clone(),
        amount: mock_amount,
        id: mock_asset_id,
        data: AssetData {
            name: String::from("name"),
            symbol: String::from("symbol"),
            decimals: 8,
        },
    };
    info!(
        "Amount of address asset before wrapping -> {:?}",
        amount_before_wrap
    );

    info!("Attempting to wrap {:?} assets", mock_amount);
    let res = TestRequest::post()
        .uri("/wrap_1155")
        .set_json(&req_data)
        .send_request(&mut app)
        .await;
    assert!(res.status().is_success(), "Failed to wrap NFT");
    //Parsing the response
    let response: ApiResponse = test::read_body_json(res).await;

    //Initializing web3
    let web3 = get_web3(&env).unwrap();
    //Waiting at least 10 confirmations to fetch the data again
    wait_for_confirmation(&web3, response.hash, ConfirmParams::with_confirmations(10))
        .await
        .expect("Failed while waiting for blockchain confirmation");

    //Fetching moralis to get address's tokens
        let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;
    let address_nft = get_nft(moralis_response.clone(), Some(mock_asset_id));
    let amount_after_wrap: u64 = address_nft.amount.clone().parse::<u64>().unwrap();
    info!(
        "Amount of address's asset after wrapping -> {:?}",
        amount_after_wrap
    );
    //Checking if the from address is the same as the transaction from address
    assert_eq!(&response.from.to_lowercase(), &mock_address.to_lowercase());
    //Checking if the amount that user's hold is what it has plus the minted amount
    let expected_token_amount = amount_before_wrap - mock_amount;
    assert_eq!(expected_token_amount, amount_after_wrap);
    
    //Print success message on terminal
    print_success("Wrap Test PASSED".to_string());
}
//Testing Unwrapper proccess
#[actix_web::test]
async fn test_4_asset_unwrap() {
    //Setup Env variables
    let env = setup_env_var();

    //Initialize rng to get random mock data
    let mut rng = rand::thread_rng();

    //Initializing testing app
    let mut app = get_app(&env).await;

    info!("\n\n Testing Unwrapper proccess");

    //Mock Data
    let mock_address: String = env.signer_address.clone();
    //Fetching moralis to get address's tokens
    let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;
    let returned_nft = get_nft(moralis_response.clone(), None);
    let mock_asset_id: u64 = returned_nft.token_id.parse::<u64>().unwrap();
    //Saving amount into a variable
    let amount_before_unwrap: u64 = returned_nft.amount.clone().parse::<u64>().unwrap();
    if amount_before_unwrap == 0 {
        panic!("Address doesn't have a nft");
    }
    let mock_amount: u64 = rng.gen_range(1..5);

    //Mocking request data (INFURA)
    //Constructing the transfer request
    let req_data = Unwrap1155 {
        recipient_address: mock_address.clone(),
        amount: mock_amount,
        id: mock_asset_id,
        data: AssetData {
            name: String::from("name"),
            symbol: String::from("symbol"),
            decimals: 8,
        },
    };
    info!(
        "Amount of address asset before unwrapping -> {:?}",
        amount_before_unwrap
    );

    info!("Attempting to unwrap {:?} assets", mock_amount);
    let res = TestRequest::post()
        .uri("/unwrap_1155")
        .set_json(&req_data)
        .send_request(&mut app)
        .await;
    assert!(res.status().is_success(), "Failed to unwrap NFT");
    //Parsing the response
    let response: ApiResponse = test::read_body_json(res).await;

    //Initializing web3
    let web3 = get_web3(&env).unwrap();
    //Waiting at least 10 confirmations to fetch the data again
    wait_for_confirmation(&web3, response.hash, ConfirmParams::with_confirmations(10))
        .await
        .expect("Failed while waiting for blockchain confirmation");

    //Fetching moralis to get address's tokens
        let moralis_response: TransactionInfo = get_address_nfts(&app, &mock_address).await;
    let address_nft = get_nft(moralis_response.clone(), Some(mock_asset_id));
    let amount_after_unwrap: u64 = address_nft.amount.clone().parse::<u64>().unwrap();
    info!(
        "Amount of address's asset after unwrapping -> {:?}",
        amount_after_unwrap
    );
    //Checking if the from address is the same as the transaction from address
    assert_eq!(&response.from.to_lowercase(), &mock_address.to_lowercase());
    //Checking if the amount that user's hold is what it has plus the unwrapped amount
    let expected_token_amount = amount_before_unwrap + mock_amount;
    assert_eq!(expected_token_amount, amount_after_unwrap);
    
    //Print success message on terminal
    print_success("Unwrap Test PASSED".to_string());
}
