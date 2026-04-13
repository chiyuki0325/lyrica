use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::config::Config;
use crate::helpers::StringVecExt;
use crate::lyric_parser::Lyric;
use crate::lyric_providers::{LyricProvider, LyricProviderError};
use crate::player::mpris_metadata::Metadata;

#[derive(Default)]
pub(crate) struct FileLyricProvider;

impl FileLyricProvider {
    fn try_parse_url(&self, metadata: &Metadata) -> Option<PathBuf> {
        let raw_url = metadata.url()?;
        if raw_url.starts_with("file://") {
            Some(Url::parse(&raw_url).ok()?.to_file_path().ok()?)
        } else {
            None
        }
    }

    fn read_tag_lyric(path: &PathBuf) -> Option<String> {
        let lower_ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match lower_ext.as_str() {
            "mp3" => Self::read_id3_tag_lyric(path),
            "flac" => Self::read_flac_tag_lyric(path),
            _ => None,
        }
    }

    fn read_id3_tag_lyric(path: &PathBuf) -> Option<String> {
        let tag = id3::Tag::read_from_path(path).ok()?;
        tag.lyrics().next().map(|lyric| lyric.text.clone())
    }

    fn read_flac_tag_lyric(path: &PathBuf) -> Option<String> {
        let tag = metaflac::Tag::read_from_path(path).ok()?;
        tag.get_vorbis("LYRICS")
            .and_then(|it| Some(it.collect::<Vec<&str>>().join("\n")))
    }

    fn read_lyric_file(path: &PathBuf) -> Option<String> {
        let lyric_path = path.with_extension("lrc");
        if lyric_path.exists() {
            std::fs::read_to_string(lyric_path).ok()
        } else {
            None
        }
    }

    async fn read_lyric_file_alt(path: &PathBuf, config: Arc<RwLock<Config>>) -> Option<String> {
        let alt_folder = config.read().await.lyric_search_folder.clone();
        let alt_path = Path::new(&alt_folder)
            .join(path.file_stem()?)
            .with_extension("lrc");
        if alt_path.exists() {
            std::fs::read_to_string(alt_path).ok()
        } else {
            None
        }
    }
}

#[async_trait]
impl LyricProvider for FileLyricProvider {
    async fn get_lyric(
        &self,
        _player_id: &str,
        metadata: &Metadata,
        config: Arc<RwLock<Config>>,
    ) -> Result<Lyric, LyricProviderError> {
        let song_path = self
            .try_parse_url(metadata)
            .ok_or(LyricProviderError::NotSupported)?;

        {
            let song_path_str = song_path.to_str().ok_or(LyricProviderError::NotSupported)?;
            if config.read().await.disabled_folders.contains(song_path_str) {
                return Err(LyricProviderError::Aborted);
                // Do not try other providers if the song is in a disabled folder
            }
        }

        let lyric_str =
            Self::read_tag_lyric(&song_path).or_else(|| Self::read_lyric_file(&song_path));

        if let Some(lyric_str) = lyric_str {
            Lyric::try_from(lyric_str).map_err(|_| LyricProviderError::NotFound)
        } else {
            Self::read_lyric_file_alt(&song_path, config)
                .await
                .ok_or(LyricProviderError::NotFound)
                .and_then(|lyric_str| {
                    Lyric::try_from(lyric_str).map_err(|_| LyricProviderError::NotFound)
                })
        }
    }
}
