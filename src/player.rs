use crate::config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;
use zbus::Connection;

pub(crate) mod dbus_proxies;
pub(crate) mod player_discovery;

pub(crate) const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
pub(crate) struct MprisListener {
    conn: Connection,
    config: Arc<RwLock<Config>>,
    players: Vec<String>,
    // stack of player names
}

impl MprisListener {
    pub(crate) async fn new(config: Arc<RwLock<Config>>) -> zbus::Result<Self> {
        let conn = Connection::session().await?;
        Ok(Self {
            conn,
            config,
            players: Vec::new(),
        })
    }
}

pub(crate) async fn start_mpris_loop(config: Arc<RwLock<Config>>) {
    // start mpris loop forever
    // if crashes or dbus connection ends, restart it after a short delay
    loop {
        let the_loop = start_mpris_loop_once(config.clone()).await;
        if let Err(e) = the_loop {
            eprintln!("Error in MPRIS loop: {}. Restarting in 5 seconds...", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn start_mpris_loop_once(config: Arc<RwLock<Config>>) -> zbus::Result<()> {
    let mut listener = MprisListener::new(config).await?;
    listener.start_discovery().await
}
