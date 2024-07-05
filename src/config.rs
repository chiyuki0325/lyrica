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
    enabled_lyric_providers: Vec<String>,
    disabled_folders: Vec<String>,
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
            "kdeconnect".to_string()
        ],
        enabled_lyric_providers: vec![
            "mpris2_text".to_string(),
            "file".to_string(),
            "yesplaymusic".to_string(),
            "feeluown_netease".to_string(),
            "netease".to_string(),
        ],
        disabled_folders: vec![],
    };
    Arc::new(RwLock::new(config))
}
