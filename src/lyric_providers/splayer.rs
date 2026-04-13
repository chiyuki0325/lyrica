use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
use serde::Deserialize;
use serde_json::{Value, from_str as from_json_str};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::Config;
use crate::lyric_parser::{Lyric, LyricLine};
use crate::lyric_providers::{LyricProvider, LyricProviderError};
use crate::player::mpris_metadata::Metadata;

const SPLAYER_DEBUG_PORT: usize = 25885;
const SPLAYER_GET_SONG_INFO: &str = "{\"type\": \"get-song-info\"}";

#[derive(Deserialize, Debug)]
struct SPlayerResponse {
    #[serde(rename = "type")]
    response_type: String,
    data: Option<SPlayerSongInfo>,
}

#[derive(Deserialize, Debug)]
struct SPlayerSongInfo {
    #[serde(rename = "lrcData")]
    lyric_data: Vec<SPlayerLyricLine>,
}

#[derive(Deserialize, Debug)]
struct SPlayerLyricLine {
    words: Vec<SPlayerLyricWord>,
    #[serde(rename = "translatedLyric")]
    translated_lyric: String,
    #[serde(rename = "startTime")]
    start_time: u64,
    #[serde(rename = "endTime")]
    end_time: u64,
}

#[derive(Deserialize, Debug)]
struct SPlayerLyricWord {
    word: String,
}

#[derive(Default)]
pub(crate) struct SPlayerLyricProvider;

impl SPlayerLyricProvider {
    fn reassemble_splayer_lyric(splayer_lyric: Vec<SPlayerLyricLine>) -> Lyric {
        let mut lyric = Lyric::new();
        for line in splayer_lyric {
            let text = line.words.into_iter().map(|w| w.word).collect::<String>();
            let alt = if line.translated_lyric.is_empty() {
                None
            } else {
                Some(line.translated_lyric)
            };
            lyric.add_line(LyricLine {
                time: line.start_time,
                text,
                alt,
            });
        }
        lyric
    }
}

#[async_trait]
impl LyricProvider for SPlayerLyricProvider {
    async fn get_lyric(
        &self,
        player_id: &str,
        _metadata: &Metadata,
        _config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        debug!("SPlayerLyricProvider: Checking player_id '{}'", player_id);
        if !player_id.starts_with("splayer") {
            return Err(LyricProviderError::NotSupported);
        }
        let url = format!("ws://localhost:{}", SPLAYER_DEBUG_PORT);
        let (ws, _) = connect_async(&url)
            .await
            .map_err(|_| {
                debug!("Failed to connect to SPlayer WebSocket at {}", url);
                LyricProviderError::NotSupported})?;

        debug!("Connected to SPlayer WebSocket at {}", url);
        
        let (mut sink, mut source) = ws.split();
        let _ = sink.send(Message::text(SPLAYER_GET_SONG_INFO)).await;

        while let Some(Ok(msg)) = source.next().await {
            if let Message::Text(text) = msg {
                if let Ok(resp) = from_json_str::<SPlayerResponse>(&text) {
                    debug!("Received SPlayer response: {:?}", resp);
                    if resp.response_type == "song-info" {
                        if let Some(data) = resp.data {
                            return Ok(Self::reassemble_splayer_lyric(data.lyric_data));
                        } else {
                            return Err(LyricProviderError::NotFound);
                        }
                    }
                }
            }
        }
        Err(LyricProviderError::NotFound)
    }
}
