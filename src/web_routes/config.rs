use actix_web::{HttpResponse, web};
use crate::config::{Config, SharedConfig};

pub(crate) async fn update_config(
    config_req: web::Json<Config>,
    config: web::Data<SharedConfig>,
) -> HttpResponse {
    if config.read().unwrap().verbose {
        println!("Updating config: {:?}", config_req.0);
    }
    let mut config = config.write().unwrap();
    *config = config_req.0;
    // check if lyric_search_folder exists
    config.alt_folder_exists = if (std::fs::metadata(&config.lyric_search_folder)).is_err() {
        false
    } else {
        true
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "ok"}"#)
}


pub(crate) async fn get_config(
    config: web::Data<SharedConfig>,
) -> HttpResponse {
    let config = config.read().unwrap();
    HttpResponse::Ok().json(&*config)
}
