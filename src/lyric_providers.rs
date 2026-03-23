use async_trait::async_trait;
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::file::FileLyricProvider;
use crate::player::mpris_metadata::Metadata;

pub(crate) mod file;

enum LyricProviderError {
    NotSupported,
    NotFound,
    Aborted,
}

#[async_trait]
trait LyricProvider: Send + Sync {
    async fn get_lyric(
        &self,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError>;
}

type LyricProviderList = Vec<Box<dyn LyricProvider>>;

lazy_static! {
    static ref LYRIC_PROVIDERS: LyricProviderList = {
        let providers: LyricProviderList = vec![Box::new(FileLyricProvider::default())];
        providers
    };
}

pub(crate) async fn try_get_lyric_from_providers(
    metadata: &Metadata,
    config: Arc<RwLock<Config>>,
) -> Option<Lyric> {
    for provider in LYRIC_PROVIDERS.iter() {
        match provider.get_lyric(metadata, config.clone()).await {
            Ok(lyric) => return Some(lyric),
            Err(LyricProviderError::Aborted) => return None,
            Err(_) => continue,
        }
    }
    None
}
