use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, fetch_netease_lyric};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct NeteaseTrackIDLyricProvider;

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
            .rsplit('/')
            .next()
            .ok_or(LyricProviderError::NotSupported)?
            .parse::<u64>()
            .map_err(|_| LyricProviderError::NotSupported)?;

        if music_id < 2 {
            // fast fail
            return Err(LyricProviderError::NotSupported);
        }
        fetch_netease_lyric(music_id, config).await
    }
}
