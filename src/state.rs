use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::Receiver;

use crate::config::Config;
use crate::messages::ChannelMessage;
pub(crate) struct State {
    // for new client connections, we need to send the current state immediately
    title: String,
    artist: String,
    text: String,
    alt: Option<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            text: String::new(),
            alt: None,
        }
    }
}

pub(crate) async fn start_manage_state(
    config: Arc<RwLock<Config>>,
    state: Arc<RwLock<State>>,
    mut rx: Receiver<ChannelMessage>,
) {
    while let Ok(msg) = rx.recv().await {
        {
            if config.read().await.verbose {
                println!("{}", &msg);
            }
        }
        match msg {
            ChannelMessage::UpdateMusicInfo(data) => {
                let mut state = state.write().await;
                state.title = data.title;
                state.artist = data.artist;
            }
            ChannelMessage::UpdateLyricLine(data) => {
                let mut state = state.write().await;
                state.text = data.text;
                state.alt = data.alt;
            }
            _ => (),
        }
    }
}
