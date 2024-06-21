pub mod file;

use std::collections::HashMap;
use lazy_static::lazy_static;
use mpris::Metadata;

pub trait LyricProvider: Sync + Send {
    fn get_lyric(&self, music_url: &str) -> (String, bool);
    fn get_lyric_by_metadata(&self, metadata: &Metadata) -> (String, bool);
    fn is_available(&self, music_url: &str) -> bool;
    fn is_meta_mode(&self) -> bool;
}


lazy_static! {
    pub static ref LYRIC_PROVIDERS: HashMap<String, Box<dyn LyricProvider>> = {
        let mut m = HashMap::new();
        m.insert("file".to_string(), Box::new(file::FileLyricProvider {}) as Box<dyn LyricProvider>);
        m
    };
}
