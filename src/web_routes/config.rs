use crate::{config::Config, messages::ChannelMessage};
use actix_web::{HttpResponse, web};
use log::{error, info};
use tokio::sync::{RwLock, broadcast};

pub(crate) async fn update_config(
    config_req: web::Json<Config>,
    config: web::Data<RwLock<Config>>,
    tx: web::Data<broadcast::Sender<ChannelMessage>>,
) -> HttpResponse {
    let new_config = config_req.into_inner();

    info!("Updating config: {:?}", &new_config);

    let mut config = config.write().await;
    *config = new_config.clone();

    // Send the updated config to all connected clients
    let channel_message = ChannelMessage::UpdateConfig(new_config);
    if let Err(e) = tx.send(channel_message) {
        error!("Failed to send config update: {}", e);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "ok"}"#)
}

pub(crate) async fn get_config(config: web::Data<RwLock<Config>>) -> HttpResponse {
    let config = config.read().await;
    HttpResponse::Ok().json(&*config)
}
