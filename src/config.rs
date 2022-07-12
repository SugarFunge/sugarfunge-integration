use ethcontract::PrivateKey;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub listen_url: String,
    pub project_id: String,
    pub signer_private_key: PrivateKey,
    pub signer_address: String,
    pub chain_id: u64,
    pub moralis_base_url: String,
    pub moralis_api_key: String,
    pub testing_address: String,
}

pub fn init() -> Config {
    let panic_message: String = "enviroment variable is not set".to_string();

    Config {
        listen_url: match env::var("LISTEN_URL") {
            Ok(var) => var,
            Err(_) => panic!("LISTEN_URL {}", panic_message),
        },
        project_id: match env::var("INFURA_PROJECT_ID") {
            Ok(var) => var,
            Err(_) => panic!("INFURA_PROJECT_ID {}", panic_message),
        },
        signer_private_key: match env::var("SIGNER_PRIVATE_KEY") {
            Ok(var) => var.parse().expect("invalid PK"),
            Err(_) => panic!("PRIVATE_KEY {}", panic_message),
        },
        signer_address: match env::var("SIGNER_ADDRESS") {
            Ok(var) => var,
            Err(_) => "".to_string(),
        },
        chain_id: match env::var("CHAIN_ID") {
            Ok(var) => var.parse::<u64>().unwrap(),
            Err(_) => panic!("CHAIN_ID {}", panic_message),
        },
        moralis_base_url: match env::var("MORALIS_BASE_URL") {
            Ok(var) => var,
            Err(_) => panic!("MORALIS_BASE_URL {}", panic_message),
        },
        moralis_api_key: match env::var("MORALIS_API_KEY") {
            Ok(var) => var,
            Err(_) => panic!("MORALIS_API_KEY {}", panic_message),
        },
        testing_address: match env::var("TESTING_ADDRESS") {
            Ok(var) => var,
            Err(_) => "".to_string(),
        },
    }
}
