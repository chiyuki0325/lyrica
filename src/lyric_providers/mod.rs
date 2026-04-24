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
use crate::lyric_cache::LYRIC_CACHE;
use crate::lyric_parser::Lyric;
use crate::player::mpris_metadata::Metadata;
use crate::helpers::StringVecExt;

use crate::lyric_providers::feeluown_netease::FeelUOwnNeteaseLyricProvider;
use crate::lyric_providers::file::FileLyricProvider;
use crate::lyric_providers::mpris2_text::Mpris2TextLyricProvider;
use crate::lyric_providers::netease::NeteaseLyricProvider;
use crate::lyric_providers::netease_trackid::NeteaseTrackIDLyricProvider;
use crate::lyric_providers::yesplaymusic::YesPlayMusicLyricProvider;
use crate::lyric_providers::splayer::SPlayerLyricProvider;


pub(crate) mod feeluown_netease;
pub(crate) mod file;
pub(crate) mod mpris2_text;
pub(crate) mod netease;
pub(crate) mod netease_trackid;
pub(crate) mod yesplaymusic;
pub(crate) mod splayer;

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
        player_id: &str,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError>;

    fn get_name(&self) -> &'static str {
        let full_name = type_name::<Self>();
        full_name.split("::").last().unwrap_or(full_name).trim_end_matches("LyricProvider")
    }
}

/// Fetch lyric from Netease by track ID, with cache read/write and retry.
async fn fetch_netease_lyric(
    music_id: u64,
    config: Arc<RwLock<Config>>,
) -> Result<Lyric, LyricProviderError> {
    let conf = config.read().await;
    let cache_enabled = conf.lyric_cache_enabled;
    let ttl_days = conf.lyric_cache_ttl_days;
    let retry = !conf.online_search_retry;
    let max_retries = conf.online_search_max_retries;
    let timeout = conf.online_search_timeout_secs;
    drop(conf);

    if cache_enabled {
        if let Some((lrc, tlyric)) = LYRIC_CACHE.get(music_id, ttl_days) {
            info!("Lyric for netease/{} loaded from disk cache", music_id);
            return Lyric::try_from((lrc, tlyric)).map_err(|_| LyricProviderError::NotFound);
        }
    }

    let music_api = use_music_api(timeout)
        .await
        .ok_or(LyricProviderError::NotSupported)?;

    let mut try_count = 0;
    while try_count < max_retries {
        info!("Fetching lyric for netease/{} from network (attempt {}/{})", music_id, try_count + 1, max_retries);
        if let Ok(result) = music_api.song_lyric(music_id).await {
            let lrc = result.lyric.join("\n");
            let tlyric = result.tlyric.join("\n");
            if cache_enabled {
                LYRIC_CACHE.put(music_id, &lrc, &tlyric);
            }
            return Lyric::try_from((lrc, tlyric)).map_err(|_| LyricProviderError::NotFound);
        }
        if retry {
            try_count += 1;
        } else {
            break;
        }
    }

    Err(LyricProviderError::Aborted)
}

type LyricProviderList = Vec<Box<dyn LyricProvider>>;

lazy_static! {
    static ref LYRIC_PROVIDERS: LyricProviderList = {
        let providers: LyricProviderList = vec![
            Box::new(Mpris2TextLyricProvider::default()),
            Box::new(FileLyricProvider::default()),
            Box::new(YesPlayMusicLyricProvider::default()),
            Box::new(NeteaseTrackIDLyricProvider::default()),
            Box::new(SPlayerLyricProvider::default()),
            Box::new(FeelUOwnNeteaseLyricProvider::default()),
            Box::new(NeteaseLyricProvider::default()),
        ];
        providers
    };
}

pub(crate) async fn try_get_lyric_from_providers(
    player_id: &str,
    metadata: &Metadata,
    config: Arc<RwLock<Config>>,
) -> Option<Lyric> {
    let enabled_providers = config.read().await.enabled_lyric_providers.clone();
    for provider in LYRIC_PROVIDERS.iter() {
        let name = provider.get_name();
        if !enabled_providers.contains(name) {
            continue;
        }
        info!("Trying lyric provider: {}", name);
        match provider.get_lyric(player_id, metadata, config.clone()).await {
            Ok(lyric) => return Some(lyric),
            Err(LyricProviderError::Aborted) => return None,
            Err(_) => continue,
        }
    }
    None
}
