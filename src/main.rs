mod websocket;
mod lyric_parser;
mod config;
mod player;
mod web_routes;
// mod lyric_providers;
mod messages;

use actix_web::{App, HttpServer, web};
use lazy_static::lazy_static;
use tokio::sync::{RwLock, broadcast};

use crate::messages::ChannelMessage;

lazy_static! {
    pub(crate) static ref PORT: u16 = 15649;
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (tx, _rx) = broadcast::channel::<ChannelMessage>(6);

    let config = config::Config::new();
    let web_data_config = web::Data::new(RwLock::new(config));

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
            .route("/ws", web::get().to(websocket::ws_index))
    })
    .bind(format!("127.0.0.1:{}", *PORT))?
    .run()
    .await
}
