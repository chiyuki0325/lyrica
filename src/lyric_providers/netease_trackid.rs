use async_trait::async_trait;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, use_music_api};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct NeteaseTrackIDLyricProvider;

impl NeteaseTrackIDLyricProvider {}

#[async_trait]
impl LyricProvider for NeteaseTrackIDLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let music_id = metadata
            .mpris_track_id()
            .ok_or(LyricProviderError::NotSupported)?
            .rsplit("/")
            .next()
            .ok_or(LyricProviderError::NotSupported)?
            .parse::<u64>()
            .map_err(|_| LyricProviderError::NotSupported)?;

        let conf = config.read().await;
        let retry = !conf.online_search_retry;
        let max_retries = conf.online_search_max_retries;
        let timeout = conf.online_search_timeout_secs;
        drop(conf);
        let mut try_count = 0;

        let music_api = use_music_api(timeout)
            .await
            .ok_or(LyricProviderError::NotSupported)?;

        while try_count < max_retries {
            info!("Trying to get lyric for track_id: {}", music_id);
            let lyric_result = music_api.song_lyric(music_id).await;
            if let Ok(lyric_result) = lyric_result {
                let lyric_lines = lyric_result.lyric;
                let tlyric_lines = lyric_result.tlyric;
                return Lyric::try_from((lyric_lines, tlyric_lines))
                    .map_err(|_| LyricProviderError::NotFound);
            } else {
                if retry {
                    try_count += 1;
                } else {
                    break;
                }
            }
        }

        Err(LyricProviderError::Aborted)
    }
}
