use async_trait::async_trait;
use isahc::HttpClient;
use isahc::prelude::Configurable;
use lazy_static::lazy_static;
use log::info;
use std::any::type_name;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OnceCell, RwLock};

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::player::mpris_metadata::Metadata;

use crate::lyric_providers::feeluown_netease::FeelUOwnNeteaseLyricProvider;
use crate::lyric_providers::file::FileLyricProvider;
use crate::lyric_providers::mpris2_text::Mpris2TextLyricProvider;
use crate::lyric_providers::netease::NeteaseLyricProvider;
use crate::lyric_providers::netease_trackid::NeteaseTrackIDLyricProvider;
use crate::lyric_providers::yesplaymusic::YesPlayMusicLyricProvider;

pub(crate) mod feeluown_netease;
pub(crate) mod file;
pub(crate) mod mpris2_text;
pub(crate) mod netease;
pub(crate) mod netease_trackid;
pub(crate) mod yesplaymusic;

lazy_static! {
    static ref MUSIC_API: OnceCell<ncm_api::MusicApi> = OnceCell::new();
}

async fn create_music_api(timeout: u64) -> Option<ncm_api::MusicApi> {
    let client = HttpClient::builder()
        .timeout(Duration::from_secs(timeout))
        .cookies()
        .build()
        .ok()?;
    Some(ncm_api::MusicApi::from_client(client))
}

async fn use_music_api(timeout: u64) -> Option<&'static ncm_api::MusicApi> {
    MUSIC_API
        .get_or_try_init(|| async { create_music_api(timeout).await.ok_or(()) })
        .await
        .ok()
}

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

    fn get_name(&self) -> &'static str {
        let full_name = type_name::<Self>();
        full_name.split("::").last().unwrap_or(full_name)
    }
}

type LyricProviderList = Vec<Box<dyn LyricProvider>>;

lazy_static! {
    static ref LYRIC_PROVIDERS: LyricProviderList = {
        let providers: LyricProviderList = vec![
            Box::new(Mpris2TextLyricProvider::default()),
            Box::new(FileLyricProvider::default()),
            Box::new(YesPlayMusicLyricProvider::default()),
            Box::new(NeteaseTrackIDLyricProvider::default()),
            Box::new(FeelUOwnNeteaseLyricProvider::default()),
            Box::new(NeteaseLyricProvider::default()),
        ];
        providers
    };
}

pub(crate) async fn try_get_lyric_from_providers(
    metadata: &Metadata,
    config: Arc<RwLock<Config>>,
) -> Option<Lyric> {
    for provider in LYRIC_PROVIDERS.iter() {
        info!("Trying lyric provider: {}", provider.get_name());
        match provider.get_lyric(metadata, config.clone()).await {
            Ok(lyric) => return Some(lyric),
            Err(LyricProviderError::Aborted) => return None,
            Err(_) => continue,
        }
    }
    None
}
