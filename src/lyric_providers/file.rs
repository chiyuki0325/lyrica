use url::Url;
use std::path::Path;
use id3::Tag as ID3Tag;
use crate::lyric_parser::{
    LyricLine,
    parse_lyrics,
};
use metaflac::Tag as FLACTag;

#[derive(Clone)]
pub struct FileLyricProvider {}

impl FileLyricProvider {
    pub async fn get_lyric(
        &self,
        music_url: &str,
        config: crate::config::SharedConfig
    ) -> (Vec<LyricLine>, bool) {
        match Self::parse_file_url(music_url) {
            Ok(path) => {
                // 此时得到了音乐文件的路径
                if let Ok(lyric) = Self::read_tag_lyric(&path) {
                    // 读取 tag 成功
                    (parse_lyrics(lyric), true)
                } else {
                    // 音乐没有 tag，直接读取 lrc
                    if let Ok(lrc) = Self::read_lrc_file(
                        &path,
                        config.read().unwrap().lyric_search_folder.clone(),
                    ) {
                        // lrc 文件存在
                        (parse_lyrics(lrc), true)
                    } else {
                        // lrc 文件也不存在
                        (Vec::new(), false)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to parse file URL: {}", e);
                (Vec::new(), false)
            }
        }
    }

    pub fn is_available(&self, music_url: &str) -> bool {
        if music_url.starts_with("file://") {
            true
        } else {
            false
        }
    }
}

impl FileLyricProvider {
    fn parse_file_url(url: &str) -> Result<String, String> {
        if url.starts_with("file://") {
            let url = Url::parse(url).map_err(|e| format!("Failed to parse URL: {}", e))?;
            let path = url.to_file_path().map_err(|_| "Failed to convert URL to file path".to_string())?;
            path.to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Failed to convert path to string".to_string())
        } else {
            Ok(url.to_string())
        }
    }

    fn read_lrc_file(
        path: &str,
        search_folder: String,
    ) -> Result<String, String> {
        let path = Path::new(path).with_extension("lrc");
        if !path.exists() {
            // search in alternative folder
            let path = Path::new(&search_folder).join(path.file_name().unwrap());
            if !path.exists() {
                return Err("Lrc file not found".to_string());
            }
        }
        std::fs::read_to_string(path.to_str().unwrap())
            .map_err(|e| format!("Failed to read file: {}", e))
    }

    fn read_tag_lyric(path: &str) -> Result<String, String> {
        match Path::new(path).extension() {
            Some(ext) => {
                let lower_ext = ext.to_str().unwrap().to_ascii_lowercase();
                if lower_ext == "mp3" {
                    Self::read_id3_tag_lyric(path)
                } else if lower_ext == "flac" {
                    Self::read_flac_tag_lyric(path)
                } else {
                    Err("Unsupported file type".to_string())
                }
            }
            None => Err("Failed to get file extension".to_string())
        }
    }

    fn read_id3_tag_lyric(path: &str) -> Result<String, String> {
        let tag = ID3Tag::read_from_path(path).map_err(|e| format!("Failed to read tag: {}", e))?;
        for lyric_tag in tag.lyrics() {
            return Ok(lyric_tag.text.clone());
        }
        Err("Lyric tag not found".to_string())
    }

    fn read_flac_tag_lyric(path: &str) -> Result<String, String> {
        if let Ok(tag) = FLACTag::read_from_path(path) {
            if let Some(lyric_tags) = tag.get_vorbis("LYRICS") {
                Ok(lyric_tags.collect::<Vec<_>>().join("\n"))
            } else {
                Err("Lyrics tag not found".to_string())
            }
        } else {
            Err("FLAC tag not found".to_string())
        }
    }
}
