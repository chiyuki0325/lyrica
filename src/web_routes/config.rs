use crate::config::{Config, SharedConfig};
use actix_web::{web, HttpResponse};

pub(crate) async fn update_config(
    config_req: web::Json<Config>,
    config: web::Data<SharedConfig>,
) -> HttpResponse {
    {
        // acquire read lock
        if config.read().await.verbose {
            println!("Updating config: {:?}", config_req.0);
        }
        // drop read lock
    }
    // acquire write lock
    let mut config = config.write().await;
    *config = config_req.0;
    // check if lyric_search_folder exists
    config.alt_folder_exists = tokio::fs::metadata(&config.lyric_search_folder).await.is_ok();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "ok"}"#)
    // all locks dropped now
}

pub(crate) async fn get_config(config: web::Data<SharedConfig>) -> HttpResponse {
    let config = config.read().await;
    HttpResponse::Ok().json(&*config)
}
