use async_trait::async_trait;
use log::{info, debug};
use serde_json::{Value, from_str as from_json_str};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError, use_music_api};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct NeteaseLyricProvider;

impl NeteaseLyricProvider {}

#[async_trait]
impl LyricProvider for NeteaseLyricProvider {
    async fn get_lyric(
        &self,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let conf = config.read().await;
        let retry = !conf.online_search_retry;
        let max_retries = conf.online_search_max_retries;
        let timeout = conf.online_search_timeout_secs;
        let pattern = conf.online_search_pattern;
        drop(conf);
        let mut try_count = 0;

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

                while try_count < max_retries {
                    let music_id = song["id"].as_u64().unwrap_or_default();
                    info!("Trying to get lyric for track_id: {}", music_id);
                    let lyric_result = music_api.song_lyric(music_id).await;
                    if let Ok(lyric_result) = lyric_result {
                        let lyric_lines = lyric_result.lyric;
                        let tlyric_lines = lyric_result.tlyric;
                        return Ok(Lyric::try_from((lyric_lines, tlyric_lines))
                            .map_err(|_| LyricProviderError::NotFound)?);
                    } else {
                        if retry {
                            try_count += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // 在线搜索都搜不到歌词，那是真没招了，直接放弃吧
        Err(LyricProviderError::Aborted)
    }
}
