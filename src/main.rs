mod test_page;
mod websocket;
mod lyric_parser;
mod player;
mod config;

use actix_web::{web, App, HttpServer, Responder};
use tokio::sync::broadcast;
use tokio::task::LocalSet;
use crate::config::{initialize_config, SharedConfig};

#[derive(Debug, Clone)]
enum ChannelMessage {
    UpdateLyricLine(i64, String),      // time, lyric
    UpdateMusicInfo(String, String),  // title, artist
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let local_set = LocalSet::new();
    // 在单线程环境中运行
    local_set
        .run_until(async move {
            let (tx, _rx) = broadcast::channel(5);
            let tx1 = tx.clone();

            let config = initialize_config();
            let config_clone = config.clone();
            let web_data_config = web::Data::new(config);


            let loop_task = tokio::task::spawn_local(async {
                player::mpris_loop(tx1, config_clone).await;
            });

            // Start the actix-web server
            HttpServer::new(move || {
                App::new()
                    .route("/test", web::get().to(test_page::test_page))
                    .app_data(web::Data::new(tx.clone()))
                    .app_data(web_data_config.clone())
                    .route("/ws", web::get().to(websocket::ws_index))
            })
                .bind("127.0.0.1:15648")?
                .run()
                .await
        })
        .await
}
