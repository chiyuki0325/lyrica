use crate::lyric_parser::{
    LyricLine,
    parse_netease_lyrics,
};

#[derive(Clone)]
pub struct FeelUOwnNeteaseLyricProvider {}


impl FeelUOwnNeteaseLyricProvider {
    pub async fn get_lyric(&self, url: &str) -> (Vec<LyricLine>, bool, bool) {
        let ncm_api = ncm_api::MusicApi::new(0);
        let music_id = url.strip_prefix("fuo://netease/songs/").unwrap().parse::<u64>().unwrap();
        let lyric_result = ncm_api.song_lyric(music_id).await;
        if let Ok(lyric_result) = lyric_result {
            let lyric_lines = lyric_result.lyric;
            let tlyric_lines = lyric_result.tlyric;
            return (
                parse_netease_lyrics(lyric_lines, tlyric_lines),
                true, false
            );
        }
        (Vec::new(), false, true)
    }

    pub fn is_available(&self, url: &str) -> bool {
        url.starts_with("fuo://netease/")
    }
}
