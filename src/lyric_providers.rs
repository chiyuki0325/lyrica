use mpris::Metadata;
use std::collections::HashMap;
use lazy_static::lazy_static;

pub mod file;
pub mod netease;

use crate::lyric_parser::LyricLine;

pub enum LyricProvider {
    File(file::FileLyricProvider),
    Netease(netease::NeteaseLyricProvider),
}

impl LyricProvider {
    pub async fn get_lyric(&self, music_url: &str, metadata: &Metadata) -> (Vec<LyricLine>, bool) {
        match self {
            LyricProvider::File(provider) => provider.get_lyric(music_url).await,
            LyricProvider::Netease(provider) => provider.get_lyric_by_metadata(metadata).await,
        }
    }

    pub fn is_available(&self, music_url: &str) -> bool {
        match self {
            LyricProvider::File(provider) => provider.is_available(music_url),
            LyricProvider::Netease(_) => true,
        }
    }
}


lazy_static! {
    pub static ref LYRIC_PROVIDERS: HashMap<&'static str, LyricProvider> = {
        let mut m = HashMap::new();
        m.insert("file", LyricProvider::File(file::FileLyricProvider {}));
        m.insert("netease", LyricProvider::Netease(netease::NeteaseLyricProvider {}));
        m
    };
}
