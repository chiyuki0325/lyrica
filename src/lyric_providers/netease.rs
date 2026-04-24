use async_trait::async_trait;
use log::debug;
use serde_json::{Value, from_str as from_json_str};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, fetch_netease_lyric, use_music_api};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct NeteaseLyricProvider;

#[async_trait]
impl LyricProvider for NeteaseLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let conf = config.read().await;
        let timeout = conf.online_search_timeout_secs;
        let pattern = conf.online_search_pattern;
        drop(conf);

        let music_api = use_music_api(timeout)
            .await
            .ok_or(LyricProviderError::NotSupported)?;

        let title = metadata.title().ok_or(LyricProviderError::NotSupported)?;
        let artist = metadata.artist().unwrap_or_else(|| String::new());

        let search_result = music_api
            .search(
                match pattern {
                    0 => format!("{} {}", title, artist),
                    1 => title.clone(),
                    _ => String::new(),
                },
                1, // 单曲
                0,
                5,
            )
            .await
            .map_err(|_| LyricProviderError::NotFound)?;

        let search_result =
            from_json_str::<Value>(&search_result).map_err(|_| LyricProviderError::NotFound)?;

        let songs = search_result["result"]["songs"]
            .as_array()
            .ok_or(LyricProviderError::NotFound)?;

        debug!("Search result songs: {:?}", songs);

        for song in songs {
            if let Some(name) = song.get("name") {
                let lower_searched_name = &name.as_str().unwrap_or_default().to_ascii_lowercase();
                let lower_metadata_name = &title.to_ascii_lowercase();
                debug!("Comparing searched name '{}' with metadata name '{}'", lower_searched_name, lower_metadata_name);

                if !(lower_searched_name.starts_with(lower_metadata_name)
                    || lower_metadata_name.starts_with(lower_searched_name))
                {
                    // 此比较方法可以使带（翻唱版）等后缀的歌曲也匹配成功
                    continue;
                }

                let searched_length =
                    Duration::from_millis(song["duration"].as_u64().unwrap_or_default());
                let metadata_length = Duration::from_micros(metadata.length().unwrap_or_default());
                debug!("searched_length: {:?}, metadata_length: {:?}", searched_length, metadata_length);

                if metadata_length
                    .checked_sub(searched_length)
                    .unwrap_or_default()
                    > Duration::from_secs(6)
                {
                    // 歌曲长度相差过大，可能匹配错误
                    continue;
                }

                let music_id = song["id"].as_u64().unwrap_or_default();
                if let Ok(lyric) = fetch_netease_lyric(music_id, config.clone()).await {
                    return Ok(lyric);
                }
            }
        }

        // 在线搜索都搜不到歌词，那是真没招了，直接放弃吧
        Err(LyricProviderError::Aborted)
    }
}
