use mpris::Metadata;
use std::collections::HashMap;
use lazy_static::lazy_static;

pub mod file;
pub mod netease;
pub mod mpris2_text;
pub mod yesplaymusic;
pub mod feeluown_netease;
pub mod netease_trackid;

use crate::lyric_parser::LyricLine;

pub enum LyricProvider {
    File(file::FileLyricProvider),
    Netease(netease::NeteaseLyricProvider),
    Mpris2Text(mpris2_text::Mpris2TextProvider),
    YesPlayMusic(yesplaymusic::YesPlayMusicLyricProvider),
    FeelUOwnNetease(feeluown_netease::FeelUOwnNeteaseLyricProvider),
    NeteaseTrackID(netease_trackid::NeteaseTrackIDLyricProvider),
}

impl LyricProvider {
    pub async fn get_lyric(
        &self,
        music_url: &str,
        metadata: &Metadata,
        online_search_pattern: u8
    ) -> (Vec<LyricLine>, bool) {
        match self {
            LyricProvider::File(provider) => provider.get_lyric(music_url).await,
            LyricProvider::Netease(provider) => provider.get_lyric_by_metadata(metadata, online_search_pattern).await,
            LyricProvider::Mpris2Text(provider) => provider.get_lyric_by_metadata(metadata).await,
            LyricProvider::YesPlayMusic(provider) => provider.get_lyric(music_url).await,
            LyricProvider::FeelUOwnNetease(provider) => provider.get_lyric(music_url).await,
            LyricProvider::NeteaseTrackID(provider) => provider.get_lyric_by_metadata(metadata).await,
        }
    }

    pub fn is_available(&self, music_url: &str, metadata: &Metadata) -> bool {
        match self {
            LyricProvider::File(provider) => provider.is_available(music_url),
            LyricProvider::Netease(_) => true,
            LyricProvider::Mpris2Text(provider) => provider.is_available_by_metadata(metadata),
            LyricProvider::YesPlayMusic(provider) => provider.  is_available(music_url, metadata),
            LyricProvider::FeelUOwnNetease(provider) => provider.is_available(music_url),
            LyricProvider::NeteaseTrackID(provider) => provider.is_available_by_metadata(metadata),
        }
    }
}


lazy_static! {
    pub static ref LYRIC_PROVIDERS: HashMap<&'static str, LyricProvider> = {
        let mut m = HashMap::new();
        m.insert("mpris2_text", LyricProvider::Mpris2Text(mpris2_text::Mpris2TextProvider {}));
        m.insert("file", LyricProvider::File(file::FileLyricProvider {}));
        m.insert("yesplaymusic", LyricProvider::YesPlayMusic(yesplaymusic::YesPlayMusicLyricProvider::new()));
        m.insert("feeluown_netease", LyricProvider::FeelUOwnNetease(feeluown_netease::FeelUOwnNeteaseLyricProvider {}));
        m.insert("netease", LyricProvider::Netease(netease::NeteaseLyricProvider {}));
        m.insert("netease_trackid", LyricProvider::NeteaseTrackID(netease_trackid::NeteaseTrackIDLyricProvider {}));
        m
    };
}
