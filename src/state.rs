use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::{Receiver, error::RecvError};

use crate::config::Config;
use crate::messages::ChannelMessage;

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    // for new client connections, we need to send the current state immediately
    pub title: String,
    pub artist: String,
    pub text: String,
    pub alt: Option<String>,
}

pub(crate) async fn start_manage_state(
    config: Arc<RwLock<Config>>,
    state: Arc<RwLock<State>>,
    mut rx: Receiver<ChannelMessage>,
) {
    loop {
        match rx.recv().await {
            Ok(msg) => {
                info!("{}", &msg);

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
                info!("State management lagged by {} messages", n);
                continue;
            }
            Err(RecvError::Closed) => {
                info!("State management channel closed");
                break;
            }
        }
    }
}
