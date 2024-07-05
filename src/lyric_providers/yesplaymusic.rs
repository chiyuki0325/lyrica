use mpris::Metadata;
use crate::lyric_parser::{
    LyricLine,
    parse_lyrics,
};
use isahc::{Request, AsyncReadResponseExt};
use isahc::HttpClient;
use serde_json::Value as JsonValue;

#[derive(Clone)]
pub struct YesPlayMusicLyricProvider {
    client: HttpClient,
}

impl YesPlayMusicLyricProvider {
    pub fn new() -> Self {
        Self {
            client: HttpClient::builder()
                .build()
                .expect("No network connection")
        }
    }

    pub async fn get_lyric(&self, music_url: &str) -> (Vec<LyricLine>, bool) {
        let req = Request::get(
            format!("http://localhost:10754/lyric?id={}", music_url.strip_prefix("/trackid/").unwrap())
        ).body(()).unwrap();

        let res = self.client.send_async(req).await;

        return if let Ok(mut res) = res {
            return if res.status().is_success() {
                let lyric: JsonValue = res.json().await.unwrap();
                let lyric_str;
                let tlyric_str;
                if let Some(lyric) = lyric.get("lrc").or(lyric.get("lyric")) {
                    lyric_str = lyric.get("lyric").unwrap().as_str().unwrap();
                } else { return (Vec::new(), false); };
                if let Some(tlyric) = lyric.get("tlyric") {
                    tlyric_str = tlyric.get("lyric").unwrap().as_str().unwrap();
                    return (parse_lyrics(format!("{}\n{}", lyric_str, tlyric_str)), true);
                } else {
                    return (parse_lyrics(lyric_str.to_string()), true);
                }
            } else { (Vec::new(), false) };
        } else { (Vec::new(), false) };
    }

    pub fn is_available(&self, url: &str, metadata: &Metadata) -> bool {
        if url.starts_with("/trackid/") {
            return if let Some(track_id) = metadata.track_id() {
                track_id.as_str().starts_with("/org/node/mediaplayer/yesplaymusic")
            } else { false };
        }
        false
    }
}
