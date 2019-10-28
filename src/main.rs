#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate diesel;

mod utils;
mod resource;
mod albatross;

use std::env;

use futures::Future;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket::State;
use rocket::request::Request;
use rocket::http::Status;
use diesel::pg::PgConnection;
use diesel::Connection;
use dotenv::dotenv;
use rand::OsRng;

use nimiq_lib::client::{ClientBuilder, Client};
use nimiq_lib::block_producer::albatross::{AlbatrossBlockProducer, ValidatorConfig};
use nimiq_network_primitives::protocol::Protocol;
use nimiq_network_primitives::services::ServiceFlags;
use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_primitives::networks::NetworkId;
use nimiq_database::lmdb::{open, LmdbEnvironment};
use nimiq_database::Environment;
use nimiq_utils::key_store::KeyStore;
use nimiq_network::network_config::Seed;
use nimiq_bls::bls12_381::KeyPair as BlsKeyPair;

use crate::resource::error::ErrorInfo;
use crate::resource::ResourceRenderer;
use crate::resource::blockchain::*;
use crate::resource::search::*;
use crate::resource::block::*;
use crate::resource::genesis::*;
use crate::resource::dashboard::*;
use crate::resource::transaction::*;
use crate::resource::error::*;
use crate::albatross::{Albatross, BlockIdentifier};
use crate::resource::metadata::MetadataStore;


const NETWORK_ID: NetworkId = NetworkId::DevAlbatross;

lazy_static! {
    static ref DB: Environment = LmdbEnvironment::new("db", 1024 * 1024 * 50, 10, open::NOMETASYNC).unwrap();
    static ref SEEDS: Vec<Seed> = vec![
        // local testnet
        //Seed::new_peer("ws://localhost:8443/53f5baf842da27a709c84c82867121355dbdf354c093d3cba79d2c339706e112".parse().unwrap()),

        // devnet
        Seed::new_peer("ws://seed1.devnet:8444/5af4c3f30998573e8d3476cd0e0543bf7adba576ef321342e41c2bccc246c377".parse().unwrap()),
    ];
}


fn main() {
    dotenv().ok();

    // init logging
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    // connect to database
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db_conn = PgConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url));
    let meta_store = MetadataStore::new(db_conn);

    // init Nimiq
    info!("Initializing Nimiq client");
    let key_store = KeyStore::new("peer_key.dat".to_string());
    let mut builder = ClientBuilder::new(Protocol::Ws, &DB, key_store);
    builder.with_network_id(NETWORK_ID);
    builder.with_hostname("explorer.nimiq.dev");
    builder.with_service_flags(ServiceFlags::FULL | ServiceFlags::VALIDATOR);
    builder.with_seeds(SEEDS.clone());
    builder.with_port(8444);
    let validator_key_store = KeyStore::new("validator_key.dat".to_string());
    let validator_config = ValidatorConfig {
        validator_key: validator_key_store.load_key().unwrap_or_else(|e| {
            /*info!("Generating validator key");
            let key_pair = BlsKeyPair::generate(&mut OsRng::new().expect("Failed to access randomness from the OS."));
            if let Err(ref err) = key_store.save_key(&key_pair) {
                warn!("Failed to save key: {}", err);
            }
            key_pair*/
            unimplemented!();
        }),
    };
    let client = builder.build_albatross_client::<AlbatrossBlockProducer>(validator_config)
        .unwrap_or_else(|e| panic!("Failed to initialize Albatross client: {}", e));
    // TODO get Arc to validator
    let albatross = Albatross::new(client.consensus());

    // init Rocket
    info!("Initializing Rocket");
    let renderer = ResourceRenderer::default();
    let rocket = rocket::ignite()
        .attach(Template::fairing())
        .manage(albatross)
        .manage(renderer)
        .register(catchers![not_found])
        .mount("/", routes![get_dashboard, get_blockchain, get_block, get_genesis, get_transaction])
        .mount("/static", StaticFiles::from("static"));

    let genesis_info = NetworkInfo::from_network_id(NetworkId::DevAlbatross);
    info!("Genesis: {}", genesis_info.genesis_hash());

    let fut = client
        .and_then(|c| c.connect())
        .map_err(|e| error!("{}", e))
        .and_then(move |_| {
            error!("rocket.launch() = {}", rocket.launch());
            Ok(())
        })
        .map_err(|e| error!("{:?}", e));

    info!("Running...");
    tokio::run(fut);
}
