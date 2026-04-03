use crate::config::Config;
use std::sync::Arc;
use log::error;
use tokio::sync::{RwLock, broadcast::Sender};
use zbus::Connection;
use crate::messages::ChannelMessage;

pub(crate) mod dbus_proxies;
pub(crate) mod mpris_loop;
pub(crate) mod mpris_metadata;
pub(crate) mod player_discovery;
pub(crate) mod player_observation;
pub(crate) mod lyric_session;

pub(crate) const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
pub(crate) struct MprisListener {
    conn: Connection,
    config: Arc<RwLock<Config>>,
    tx: Sender<ChannelMessage>,
}

impl MprisListener {
    pub(crate) async fn new(config: Arc<RwLock<Config>>, tx: Sender<ChannelMessage>) -> zbus::Result<Self> {
        let conn = Connection::session().await?;
        Ok(Self { conn, config, tx })
    }
}

pub(crate) async fn start_mpris_loop(
    config: Arc<RwLock<Config>>,
    tx: Sender<ChannelMessage>,
) {
    // start mpris loop forever
    // if crashes or dbus connection ends, restart it after a short delay
    loop {
        let the_loop = start_mpris_loop_once(config.clone(), tx.clone()).await;
        if let Err(e) = the_loop {
            error!("Error in MPRIS loop: {}. Restarting in 5 seconds...", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn start_mpris_loop_once(config: Arc<RwLock<Config>>, tx: Sender<ChannelMessage>) -> zbus::Result<()> {
    let listener = MprisListener::new(config, tx).await?;
    listener.start_mpris_loop().await
}
