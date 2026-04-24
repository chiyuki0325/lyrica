use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, fetch_netease_lyric};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct FeelUOwnNeteaseLyricProvider;

#[async_trait]
impl LyricProvider for FeelUOwnNeteaseLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let url = metadata.url().ok_or(LyricProviderError::NotSupported)?;

        if !url.starts_with("fuo://netease/") {
            return Err(LyricProviderError::NotSupported);
        }

        let music_id = url
            .strip_prefix("fuo://netease/songs/")
            .ok_or(LyricProviderError::NotSupported)?
            .parse::<u64>()
            .map_err(|_| LyricProviderError::NotSupported)?;

        fetch_netease_lyric(music_id, config).await
    }
}
