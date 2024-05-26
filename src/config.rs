use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use crate::config;

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
    disabled_players: Vec<String>,
    enabled_lyric_providers: Vec<String>,
    provider_settings: ProviderSettings,
    disabled_folders: Vec<String>,
});

pub_struct!(ProviderSettings {
    netease: NetEaseProviderSettings,
    yesplaymusic: YesPlayMusicProviderSettings,
});

pub_struct!(NetEaseProviderSettings {
    prefer_tlyric: bool,
});

pub_struct!(YesPlayMusicProviderSettings {
    prefer_tlyric: bool,
});


pub type SharedConfig = Arc<RwLock<Config>>;

pub fn initialize_config() -> SharedConfig {
    let config = config::Config {
        verbose: true,
        disabled_players: vec![
            "firefox".to_string(),
            "chromium".to_string(),
            "plasma-browser-integration".to_string(),
            "kdeconnect".to_string()
        ],
        enabled_lyric_providers: vec![
            "file".to_string(),
            "netease".to_string(),
            "yesplaymusic".to_string()
        ],
        provider_settings: config::ProviderSettings {
            netease: config::NetEaseProviderSettings {
                prefer_tlyric: false,
            },
            yesplaymusic: config::YesPlayMusicProviderSettings {
                prefer_tlyric: false,
            },
        },
        disabled_folders: vec![],
    };
    Arc::new(RwLock::new(config))
}
