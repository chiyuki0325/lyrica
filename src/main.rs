mod config;
mod helpers;
mod lyric_cache;
mod lyric_parser;
mod lyric_providers;
mod messages;
mod player;
mod state;
mod web_routes;
mod websocket;

use std::time::Duration;

use actix_web::{App, HttpServer, web};
use lazy_static::lazy_static;
use tokio::sync::{RwLock, broadcast};

use crate::lyric_cache::LYRIC_CACHE;

use crate::config::Config;
use crate::messages::ChannelMessage;
use crate::player::start_mpris_loop;
use crate::state::{State, start_manage_state};

lazy_static! {
    pub(crate) static ref PORT: u16 = 15650;
    // TODO: make this configurable in cmd args
}

// multithreaded runtime is too heavy so we use single-threaded runtime
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    // initialize logging
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .init();

    let (tx, rx) = broadcast::channel::<ChannelMessage>(6);

    let config = Config::new();
    let web_data_config = web::Data::new(RwLock::new(config));

    let state = State::default();
    let web_data_state = web::Data::new(RwLock::new(state));

    // start mpris loop in background task
    tokio::spawn(start_mpris_loop(
        web_data_config.clone().into_inner(),
        tx.clone(),
    ));

    // start lyric cache cleanup background task
    let cache_config = web_data_config.clone().into_inner();
    tokio::spawn(async move {
        loop {
            let ttl = cache_config.read().await.lyric_cache_ttl_days;
            LYRIC_CACHE.cleanup_expired(ttl);
            tokio::time::sleep(Duration::from_secs(86400)).await;
        }
    });

    // start state management in background task
    tokio::spawn(start_manage_state(
        web_data_config.clone().into_inner(),
        web_data_state.clone().into_inner(),
        rx,
    ));

    println!("Lyrica is running at port {}", *PORT);

    // Start the actix-web server
    HttpServer::new(move || {
        App::new()
            .route("/test", web::get().to(web_routes::test_page::test_page))
            .route("/config", web::get().to(web_routes::config::get_config))
            .route(
                "/config/update",
                web::post().to(web_routes::config::update_config),
            )
            .app_data(web::Data::new(tx.clone()))
            .app_data(web_data_config.clone())
            .app_data(web_data_state.clone())
            .route("/ws", web::get().to(websocket::ws_index))
    })
    .workers(1)
    .bind(format!("127.0.0.1:{}", *PORT))?
    .run()
    .await
}
