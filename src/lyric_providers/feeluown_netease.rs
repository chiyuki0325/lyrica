use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, use_music_api};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct FeelUOwnNeteaseLyricProvider;

#[async_trait]
impl LyricProvider for FeelUOwnNeteaseLyricProvider {
    async fn get_lyric(
        &self,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let url = metadata.url().ok_or(LyricProviderError::NotSupported)?;

        if url.starts_with("fuo://netease/") {
            let timeout = config.read().await.online_search_timeout_secs;
            let music_api = use_music_api(timeout).await.ok_or(LyricProviderError::NotSupported)?;
            let music_id = url
                .strip_prefix("fuo://netease/songs/")
                .unwrap()
                .parse::<u64>()
                .unwrap();
            let lyric_result = music_api.song_lyric(music_id).await;
            if let Ok(lyric_result) = lyric_result {
                let lyric_lines = lyric_result.lyric;
                let tlyric_lines = lyric_result.tlyric;
                Lyric::try_from((lyric_lines, tlyric_lines))
                    .map_err(|_| LyricProviderError::NotFound)
            } else {
                Err(LyricProviderError::NotFound)
            }
        } else {
            return Err(LyricProviderError::NotSupported);
        }
    }
}
