mod test_page;
mod websocket;
mod lyric_parser;
mod mpris;

use actix_web::{web, App, HttpServer, Responder};
use tokio::time::{sleep, Duration};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
enum ChannelMessage {
    UpdateLyricLine(String),
    UpdateMusicInfo(String, String),
}


async fn loop_function(tx: broadcast::Sender<ChannelMessage>) {
    loop {
        println!("This is a loop function running in the background.");
        tx.send(ChannelMessage::UpdateLyricLine("QWQ Hello, world!".to_string())).unwrap();
        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (tx, _rx) = broadcast::channel(5);

    // Spawn the loop function as a background task
    let tx2 = tx.clone();
    let tx1 = tx.clone();
    tokio::spawn(async {
        loop_function(tx2).await;
    });

    // Start the actix-web server
    HttpServer::new(move || {
        App::new()
            .route("/test", web::get().to(test_page::test_page))
            .app_data(web::Data::new(tx1.clone()))
            .route("/ws", web::get().to(websocket::ws_index))
    })
        .bind("127.0.0.1:15648")?
        .run()
        .await
}
