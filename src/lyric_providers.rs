use mpris::Metadata;
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
        config: crate::config::SharedConfig,
    ) -> (Vec<LyricLine>, bool) {
        match self {
            LyricProvider::File(provider) => provider.get_lyric(music_url, config).await,
            LyricProvider::Netease(provider) => provider.get_lyric_by_metadata(metadata, config).await,
            LyricProvider::Mpris2Text(provider) => provider.get_lyric_by_metadata(metadata).await,
            LyricProvider::YesPlayMusic(provider) => provider.get_lyric(music_url).await,
            LyricProvider::FeelUOwnNetease(provider) => provider.get_lyric(music_url).await,
            LyricProvider::NeteaseTrackID(provider) => provider.get_lyric_by_metadata(metadata, config).await,
        }
    }

    pub fn is_available(&self, music_url: &str, metadata: &Metadata) -> bool {
        match self {
            LyricProvider::File(provider) => provider.is_available(music_url),
            LyricProvider::Netease(_) => true,
            LyricProvider::Mpris2Text(provider) => provider.is_available_by_metadata(metadata),
            LyricProvider::YesPlayMusic(provider) => provider.is_available(music_url, metadata),
            LyricProvider::FeelUOwnNetease(provider) => provider.is_available(music_url),
            LyricProvider::NeteaseTrackID(provider) => provider.is_available_by_metadata(metadata),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            LyricProvider::File(_) => "File",
            LyricProvider::Netease(_) => "Netease",
            LyricProvider::Mpris2Text(_) => "Mpris2Text",
            LyricProvider::YesPlayMusic(_) => "YesPlayMusic",
            LyricProvider::FeelUOwnNetease(_) => "FeelUOwnNetease",
            LyricProvider::NeteaseTrackID(_) => "NeteaseTrackID",
        }
    }
}


lazy_static! {
    pub static ref LYRIC_PROVIDERS: Vec<LyricProvider> = vec![
        LyricProvider::Mpris2Text(mpris2_text::Mpris2TextProvider {}),  // 0
        LyricProvider::File(file::FileLyricProvider {}),  // 1
        LyricProvider::YesPlayMusic(yesplaymusic::YesPlayMusicLyricProvider::new()),  // 2
        LyricProvider::NeteaseTrackID(netease_trackid::NeteaseTrackIDLyricProvider {}),  // 3
        LyricProvider::FeelUOwnNetease(feeluown_netease::FeelUOwnNeteaseLyricProvider {}),  // 4
        LyricProvider::Netease(netease::NeteaseLyricProvider {}),  // 5
    ];
}
