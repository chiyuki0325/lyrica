use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use isahc::{AsyncReadResponseExt, HttpClient, Request};
use serde_json::Value as JsonValue;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError};
use crate::player::mpris_metadata::Metadata;

pub(crate) struct YesPlayMusicLyricProvider {
    client: Option<HttpClient>,
}

impl Default for YesPlayMusicLyricProvider {
    fn default() -> Self {
        Self {
            client: HttpClient::new().ok(),
        }
    }
}

impl YesPlayMusicLyricProvider {
    async fn get_lyric(&self, track_id: &str) -> Result<Lyric, LyricProviderError> {
        let http_client = self.client.as_ref().ok_or(LyricProviderError::NotSupported)?;

        let req = Request::get(format!("http://localhost:10754/lyric?id={}", track_id))
            .body(())
            .unwrap();

        let res = http_client.send_async(req).await;

        if let Ok(mut res) = res
            && res.status().is_success()
        {
            let lyric: JsonValue = res.json().await.unwrap();
            let lyric_str;
            let tlyric_str;
            if let Some(lyric) = lyric.get("lrc").or(lyric.get("lyric")) {
                lyric_str = lyric.get("lyric").unwrap().as_str().unwrap();
            } else {
                return Err(LyricProviderError::NotFound);
            };
            if let Some(tlyric) = lyric.get("tlyric") {
                tlyric_str = tlyric.get("lyric").unwrap().as_str().unwrap();
                Lyric::try_from((lyric_str.to_string(), tlyric_str.to_string()))
                    .map_err(|_| LyricProviderError::NotFound)
            } else {
                Lyric::try_from((lyric_str.to_string(), String::new()))
                    .map_err(|_| LyricProviderError::NotFound)
            }
        } else {
            Err(LyricProviderError::NotFound)
        }
    }
}

#[async_trait]
impl LyricProvider for YesPlayMusicLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        _config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        // TODO: judge player_id to make sure it's YesPlayMusic
        let url = metadata.url().ok_or(LyricProviderError::NotSupported)?;
        if url.starts_with("/trackid/") {
            if let Some(track_id) = metadata.mpris_track_id() {
                if track_id
                    .as_str()
                    .starts_with("/org/node/mediaplayer/yesplaymusic")
                {
                    return self.get_lyric(track_id.as_str()).await;
                }
            }
        }
        Err(LyricProviderError::NotSupported)
    }
}
