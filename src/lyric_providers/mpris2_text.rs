use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct Mpris2TextLyricProvider;

#[async_trait]
impl LyricProvider for Mpris2TextLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        _config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        if let Some(mpris2_text) = metadata.xesam_as_text() {
            Ok(Lyric::try_from(mpris2_text).map_err(|_| LyricProviderError::NotFound)?)
        } else {
            Err(LyricProviderError::NotFound)
        }
    }
}
