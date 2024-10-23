use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

// source: https://stackoverflow.com/questions/53866508
macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // ewww
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(Config {
    verbose: bool,
    tlyric_mode: u8,
    disabled_players: Vec<String>,
    enabled_lyric_providers: Vec<usize>,
    online_search_pattern: u8,
    disabled_folders: Vec<String>,
    online_search_timeout: u64,
    online_search_retry: bool,
    max_retries: u8,
    lyric_search_folder: String,
    alt_folder_exists: bool,
});


pub type SharedConfig = Arc<RwLock<Config>>;

pub fn initialize_config() -> SharedConfig {
    let config = Config {
        verbose: true,
        tlyric_mode: 1,
        // 0: always use original lyric
        // 1: show tlyric instead of lyric if available
        // 2: Lyric | TLyric
        // 3: TLyric | Lyric
        disabled_players: vec![
            "firefox".to_string(),
            "chromium".to_string(),
            "plasma-browser-integration".to_string(),
            "kdeconnect".to_string(),
        ],
        enabled_lyric_providers: vec![0, 1, 2, 3, 4, 5],
        online_search_pattern: 0,
        // 0: Title + Artist
        // 1: Title only
        disabled_folders: vec![],
        online_search_timeout: 10,
        online_search_retry: true,
        max_retries: 3,
        lyric_search_folder: "~/Music/lrc".to_string(),
        alt_folder_exists: false,
    };
    Arc::new(RwLock::new(config))
}
