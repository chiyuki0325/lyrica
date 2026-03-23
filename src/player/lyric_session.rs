use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::player::MPRIS_PREFIX;
use crate::{lyric_parser::Lyric, player::player_observation::PlaybackEvent};

pub(crate) struct SessionManager {
    player_id: String,
    service_name: String,
    config: Arc<RwLock<Config>>,
    handle: Option<JoinHandle<()>>,
    tx: Sender<PlaybackEvent>,
}

impl SessionManager {
    pub async fn new(
        player_id: String,
        config: Arc<RwLock<Config>>,
        tx: Sender<PlaybackEvent>,
    ) -> Self {
        Self {
            player_id: player_id.clone(),
            service_name: format!("{}{}", MPRIS_PREFIX, player_id),
            config: config,
            handle: None,
            tx: tx,
        }
    }
    pub async fn start_session(&mut self, lyric: Lyric) {
        // terminate existing session if exists
        self.handle.take().map(|h| h.abort());

        // start new session
        let mut rx = self.tx.subscribe();
        let config = self.config.clone();
        self.handle = Some(tokio::spawn(async move {
            Self::start_lyric_session(config, lyric, rx).await;
        }));
    }

    pub(crate) async fn start_lyric_session(
        config: Arc<RwLock<Config>>,
        lyric: Lyric,
        mut rx: Receiver<PlaybackEvent>,
    ) {
        // TODO
        while let Ok(event) = rx.recv().await {
            println!("Received playback event in lyric session: {:?}", event);
        }
    }

    pub(crate) fn send(&self, event: PlaybackEvent) {
        self.tx.send(event).ok();
    }
}
