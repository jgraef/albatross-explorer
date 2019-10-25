#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

mod utils;
mod resources;
mod albatross;

use futures::Future;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket::State;
use rocket::request::Request;
use rocket::http::Status;

use nimiq_lib::client::{ClientBuilder, Client};
use nimiq_lib::block_producer::DummyBlockProducer;
use nimiq_network_primitives::protocol::Protocol;
use nimiq_network_primitives::services::ServiceFlags;
use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_primitives::networks::NetworkId;
use nimiq_database::lmdb::{open, LmdbEnvironment};
use nimiq_database::Environment;
use nimiq_utils::key_store::KeyStore;
use nimiq_network::network_config::Seed;

use crate::resources::error::ErrorInfo;
use crate::resources::ResourceRenderer;
use crate::resources::dashboard::*;
use crate::albatross::{Albatross, BlockIdentifier};


const NETWORK_ID: NetworkId = NetworkId::DevAlbatross;

lazy_static! {
    static ref DB: Environment = LmdbEnvironment::new("db", 1024 * 1024 * 50, 10, open::NOMETASYNC).unwrap();
    static ref SEEDS: Vec<Seed> = vec![
        // local testnet
        //Seed::new_peer("ws://localhost:8443/53f5baf842da27a709c84c82867121355dbdf354c093d3cba79d2c339706e112".parse().unwrap()),

        // devnet
        Seed::new_peer("ws://144.76.172.80:8443/5af4c3f30998573e8d3476cd0e0543bf7adba576ef321342e41c2bccc246c377".parse().unwrap()),
    ];
}



#[catch(404)]
fn not_found(request: &Request) -> Template {
    let renderer = request.guard::<State<ResourceRenderer>>().expect("Missing renderer");
    renderer.render("error", ErrorInfo::from(Status::NotFound))
}

#[get("/genesis-info")]
fn get_genesis_info(albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Template {
    renderer.render("genesis", &albatross.genesis_info)
}


#[get("/block/<ident>")]
fn get_block(ident: BlockIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    Some(renderer.render("block", albatross.get_block_info(&ident)))
}


fn main() {
    // init logging
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // init Nimiq
    let key_store = KeyStore::new("peer_key.dat".to_string());
    let mut builder = ClientBuilder::new(Protocol::Ws, &DB, key_store);
    builder.with_network_id(NETWORK_ID);
    builder.with_hostname("explorer.nimiq.dev");
    builder.with_service_flags(ServiceFlags::FULL | ServiceFlags::VALIDATOR);
    builder.with_seeds(SEEDS.clone());
    builder.with_port(8444);
    let client = builder.build_albatross_client::<DummyBlockProducer>(())
        .unwrap_or_else(|e| panic!("Failed to initialize Albatross client: {}", e));
    let albatross = Albatross::new(client.consensus());

    // init Rocket
    let renderer = ResourceRenderer::default();
    let rocket = rocket::ignite()
        .attach(Template::fairing())
        .manage(albatross)
        .manage(renderer)
        .register(catchers![not_found])
        .mount("/", routes![get_dashboard, get_block])
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

    tokio::run(fut);
}
