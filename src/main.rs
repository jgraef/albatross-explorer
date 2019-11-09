#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate nimiq_macros;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate diesel;
extern crate nimiq_lib2 as nimiq;

mod utils;
mod resource;
mod albatross;
mod schema;

use std::env;

use futures::Future;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use dotenv::dotenv;

/*use nimiq_network_primitives::protocol::Protocol;
use nimiq_network_primitives::services::ServiceFlags;
use nimiq_database::lmdb::{open, LmdbEnvironment};
use nimiq_database::Environment;
use nimiq_utils::key_store::KeyStore;
use nimiq_network::network_config::Seed;*/

use nimiq::prelude::*;
use nimiq::config::consts::WS_DEFAULT_PORT;
use nimiq::config::config::FileStorageConfig;
use nimiq_primitives::networks::NetworkId;
use nimiq_network_primitives::networks::NetworkInfo;
use nimiq::extras::deadlock::initialize_deadlock_detection;

use crate::resource::ResourceRenderer;
use crate::resource::blockchain::*;
use crate::resource::search::*;
use crate::resource::block::*;
use crate::resource::genesis::*;
use crate::resource::dashboard::*;
use crate::resource::transaction::*;
use crate::resource::error::*;
use crate::resource::account::*;
use crate::albatross::{Albatross, BlockIdentifier};
use crate::resource::metadata::MetadataStore;
use std::path::Path;


const NETWORK_ID: NetworkId = NetworkId::DevAlbatross;


fn main() {
    dotenv().ok();

    // init logging
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // initialize deadlock detection
    initialize_deadlock_detection();

    tokio::run(futures::future::lazy(move || {
        // connect to database
        let db_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let meta_store = MetadataStore::new(db_url);

        // init Nimiq
        let client: Client = ClientConfig::builder()
            .network(NETWORK_ID)
            .full()
            .ws("explorer.albatross.nimiq.dev", WS_DEFAULT_PORT + 1)
            .storage(FileStorageConfig::from_directory("./data"))
            .validator()
            .instantiate_client()
            .expect("Failed to configure Nimiq");

        // build albatross object, which also manages meta-data
        let albatross = Albatross::new(client.clone(), meta_store);

        // start Nimiq
        client.initialize().unwrap();
        client.connect().unwrap();

        // init Rocket
        info!("Initializing Rocket");
        let renderer = ResourceRenderer::default();
        rocket::ignite()
            .attach(Template::fairing())
            .manage(albatross)
            .manage(renderer)
            .register(catchers![not_found])
            .mount("/", routes![
                get_dashboard,
                get_blockchain,
                get_block,
                download_block,
                get_genesis,
                get_transaction,
                download_transaction,
                get_accounts,
                get_account,
                get_search,
            ])
            .mount("/static", StaticFiles::from("static"))
            .launch();

        Ok(())
    }));
}
