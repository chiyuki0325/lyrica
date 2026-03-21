use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub verbose: bool,
    pub disabled_players: Vec<String>,
    pub enabled_lyric_providers: Vec<usize>,
    pub online_search_pattern: u8,
    pub disabled_folders: Vec<String>,
    pub online_search_timeout: u64,
    pub online_search_retry: bool,
    pub online_search_max_retries: u8,
    pub lyric_search_folder: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            verbose: false,
            disabled_players: "firefox,chromium,plasma-browser-integration,kdeconnect"
                .split(',')
                .map(|it| it.trim().to_string())
                .collect(),
            enabled_lyric_providers: vec![0, 1, 2, 3, 4, 5],
            online_search_pattern: 0,
            // 0: Title + Artist
            // 1: Title only
            disabled_folders: vec![],
            online_search_timeout: 10,
            online_search_retry: true,
            online_search_max_retries: 3,
            lyric_search_folder: "~/Music/lrc".to_string(),
        }
    }
}
