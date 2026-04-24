use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub disabled_players: Vec<String>,
    pub enabled_lyric_providers: Vec<String>,
    pub online_search_pattern: u8,
    pub disabled_folders: Vec<String>,
    pub online_search_timeout_secs: u64,
    pub online_search_retry: bool,
    pub online_search_max_retries: u8,
    pub lyric_search_folder: String,
    pub lyric_cache_enabled: bool,
    pub lyric_cache_ttl_days: u32,
}

impl Config {
    pub fn new() -> Self {
        Self {
            disabled_players: "firefox,chromium,plasma-browser-integration,kdeconnect"
                .split(',')
                .map(|it| it.trim().to_string())
                .collect(),
            enabled_lyric_providers: "Mpris2Text,File,NeteaseTrackID,YesPlayMusic,SPlayer,FeelUOwnNetease,Netease"
                .split(',')
                .map(|it| it.trim().to_string())
                .collect(),
            online_search_pattern: 0,
            // 0: Title + Artist
            // 1: Title only
            disabled_folders: vec![],
            online_search_timeout_secs: 10,
            online_search_retry: true,
            online_search_max_retries: 3,
            lyric_search_folder: "~/Music/lrc".to_string(),
            lyric_cache_enabled: true,
            lyric_cache_ttl_days: 30,
        }
    }
}
