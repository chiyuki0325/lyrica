use mpris::Metadata;
use serde_json::{
    Value,
    from_str as from_json_str,
};
use std::time::Duration;
use crate::lyric_parser::{
    LyricLine,
    parse_netease_lyrics,
};

#[derive(Clone)]
pub struct NeteaseLyricProvider {}


impl NeteaseLyricProvider {
    pub async fn get_lyric_by_metadata(
        &self,
        metadata: &Metadata,
        online_search_pattern: u8
    ) -> (Vec<LyricLine>, bool) {
        let ncm_api = ncm_api::MusicApi::new(0);
        let title = metadata.title().unwrap_or_default().to_string();
        let artist = metadata.artists().unwrap_or_default().get(0).unwrap_or(&"").to_string();
        let search_result = ncm_api.search(
            match online_search_pattern{
                0 => title + " " + &artist,
                1 => title,
                _ => String::new(),
            },
            1, // 单曲
            0,
            5,
        ).await;
        return if let Ok(search_result) = search_result {
            // 搜索有结果
            let search_result: Value = from_json_str(&search_result).unwrap();
            for song in search_result["result"]["songs"].as_array().unwrap_or(&Vec::new()) {
                if let Some(name) = song.get("name") {
                    if name.as_str().unwrap_or_default().to_ascii_lowercase().starts_with(
                        metadata.title().unwrap_or_default().to_ascii_lowercase().as_str()
                        // 此比较方法可以使带（翻唱版）等后缀的歌曲也匹配成功
                    ) {
                        let searched_length = Duration::from_millis(song["duration"].as_u64().unwrap());
                        let music_length = metadata.length().unwrap_or_default();
                        if music_length.checked_sub(searched_length).unwrap_or_default() < Duration::from_secs(6) {
                            // 相差不超过 6 秒
                            let lyric_result = ncm_api.song_lyric(song["id"].as_u64().unwrap()).await;
                            if let Ok(lyric_result) = lyric_result {
                                let lyric_lines = lyric_result.lyric;
                                let tlyric_lines = lyric_result.tlyric;
                                return (
                                    parse_netease_lyrics(lyric_lines, tlyric_lines),
                                    true
                                );
                            }
                        }
                    }
                }
            }
            (Vec::new(), false)
        } else {
            // 搜索没结果
            (Vec::new(), false)
        };
    }
}
