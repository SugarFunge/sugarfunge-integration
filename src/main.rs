use actix_cors::Cors;
use actix_web::{http, web::Data, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sugarfunge_integration::asset::*;
use sugarfunge_integration::config;
use sugarfunge_integration::moralis::*;
use sugarfunge_integration::wrapper::*;
#[derive(Serialize, Deserialize, Debug)]
enum ContentType {
    JSON,
}

impl ContentType {
    #[warn(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match *self {
            ContentType::JSON => "application/json",
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let env = config::init();

    let url = env.listen_url.to_owned();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(prometheus.clone())
            .wrap(cors)
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
            .app_data(Data::new(env.clone()))
    })
    .bind(url)?
    .run()
    .await
}
