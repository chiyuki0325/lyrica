use mpris::Metadata;
use crate::lyric_parser::{
    LyricLine,
    parse_lyrics,
};

#[derive(Clone)]
pub struct Mpris2TextProvider {}

impl Mpris2TextProvider {
    pub async fn get_lyric_by_metadata(&self, metadata: &Metadata) -> (Vec<LyricLine>, bool, bool) {
        if let Some(mpris2_text) = metadata.get("xesam:asText") {
            let mpris2_text = mpris2_text.as_string().unwrap().clone();
            if mpris2_text.lines().all(|line| line.starts_with('[')) {
                (parse_lyrics(mpris2_text), true, false)
            } else {
                (Vec::new(), false, true)
            }
        } else {
            (Vec::new(), false, true)
        }
    }

    pub fn is_available_by_metadata(&self, metadata: &Metadata) -> bool {
        metadata.iter().any(|(k, _)| k == "xesam:asText")
    }
}
