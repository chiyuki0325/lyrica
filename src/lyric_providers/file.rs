use mpris::Metadata;
use crate::lyric_providers::LyricProvider;
use url::Url;
use std::path::Path;
use std::fs::File;
use id3::Tag;
use flac::metadata;

#[derive(Clone)]
pub struct FileLyricProvider {}

impl LyricProvider for FileLyricProvider {
    fn get_lyric(&self, music_url: &str) -> (String, bool) {
        match Self::parse_file_url(music_url) {
            Ok(path) => {
                // 此时得到了音乐文件的路径
                if let Ok(lyric) = Self::read_tag_lyric(&path) {
                    // 读取 tag 成功
                    (lyric, true)
                } else {
                    // 音乐没有 tag，直接读取 lrc
                    if let Ok(lrc) = Self::read_lrc_file(&path) {
                        // lrc 文件存在
                        (lrc, true)
                    } else {
                        // lrc 文件也不存在
                        (String::new(), false)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to parse file URL: {}", e);
                (String::new(), false)
            }
        }
    }

    fn get_lyric_by_metadata(&self, metadata: &Metadata) -> (String, bool) {
        // 未实现
        (String::new(), false)
    }

    fn is_available(&self, music_url: &str) -> bool {
        if music_url.starts_with("file://") {
            true
        } else {
            false
        }
    }

    fn is_meta_mode(&self) -> bool {
        // 是否需要 metadata
        false
    }
}

impl FileLyricProvider {
    pub fn parse_file_url(url: &str) -> Result<String, String> {
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

    pub fn read_lrc_file(path: &str) -> Result<String, String> {
        let mut path = Path::new(path).with_extension("lrc");
        if !path.exists() {
            return Err("Lrc file not found".to_string());
        }
        std::fs::read_to_string(path.to_str().unwrap())
            .map_err(|e| format!("Failed to read file: {}", e))
    }

    pub fn read_tag_lyric(path: &str) -> Result<String, String> {
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

    pub fn read_id3_tag_lyric(path: &str) -> Result<String, String> {
        let tag = Tag::read_from_path(path).map_err(|e| format!("Failed to read tag: {}", e))?;
        for lyric_tag in tag.lyrics() {
            return Ok(lyric_tag.text.clone());
        }
        Err("Lyric tag not found".to_string())
    }

    pub fn read_flac_tag_lyric(path: &str) -> Result<String, String> {
        if let Ok(vorbis_comment) = metadata::get_vorbis_comment(path) {
            for comment in vorbis_comment.comments {
                if comment.0.to_uppercase() == "LYRICS" {
                    return Ok(comment.1.clone());
                }
            }
        }
        Err("FLAC tag not found".to_string())
    }
}
