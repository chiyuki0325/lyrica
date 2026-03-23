use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::{Receiver, error::RecvError};

use crate::config::Config;
use crate::messages::ChannelMessage;

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    // for new client connections, we need to send the current state immediately
    title: String,
    artist: String,
    text: String,
    alt: Option<String>,
}

pub(crate) async fn start_manage_state(
    config: Arc<RwLock<Config>>,
    state: Arc<RwLock<State>>,
    mut rx: Receiver<ChannelMessage>,
) {
    loop {
        match rx.recv().await {
            Ok(msg) => {
                let is_verbose = config.read().await.verbose;
                if is_verbose {
                    println!("{}", &msg);
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
            Err(RecvError::Lagged(n)) => {
                continue;
            }
            Err(RecvError::Closed) => {
                break;
            }
        }
    }
}
