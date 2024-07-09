use mpris::Metadata;
use crate::lyric_parser::{
    LyricLine,
    parse_netease_lyrics,
};

#[derive(Clone)]
pub struct NeteaseTrackIDLyricProvider {}


impl NeteaseTrackIDLyricProvider {
    pub async fn get_lyric_by_metadata(&self, metadata: &Metadata) -> (Vec<LyricLine>, bool) {
        let ncm_api = ncm_api::MusicApi::new(0);
        let track_id = metadata.track_id().unwrap().to_string();
        let music_id = track_id.rsplit("/").next().unwrap().parse::<u64>().unwrap();
        let lyric_result = ncm_api.song_lyric(music_id).await;
        if let Ok(lyric_result) = lyric_result {
            let lyric_lines = lyric_result.lyric;
            let tlyric_lines = lyric_result.tlyric;
            return (
                parse_netease_lyrics(lyric_lines, tlyric_lines),
                true
            );
        }
        (Vec::new(), false)
    }

    pub fn is_available_by_metadata(&self, metadata: &Metadata) -> bool {
        metadata.track_id().is_some() && metadata.track_id().unwrap().as_str().rspace("/").next().unwrap().parse::<u64>().is_ok()
    }
}
