use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    lrc: String,
    tlyric: String,
    cached_at: u64,
}

pub(crate) struct LyricCache {
    cache_dir: PathBuf,
}

fn resolve_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("LYRICA_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("lyrica");
    }
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".cache")
        .join("lyrica")
}

impl LyricCache {
    fn new() -> Self {
        Self {
            cache_dir: resolve_cache_dir(),
        }
    }

    fn netease_dir(&self) -> PathBuf {
        self.cache_dir.join("netease")
    }

    fn cache_path(&self, track_id: u64) -> PathBuf {
        self.netease_dir().join(format!("{}.json", track_id))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    pub fn get(&self, track_id: u64, ttl_days: u32) -> Option<(String, String)> {
        let content = std::fs::read_to_string(self.cache_path(track_id)).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        if ttl_days > 0 {
            let age = Self::now_secs().saturating_sub(entry.cached_at);
            if age > ttl_days as u64 * 86400 {
                return None;
            }
        }

        Some((entry.lrc, entry.tlyric))
    }

    pub fn put(&self, track_id: u64, lrc: &str, tlyric: &str) {
        if std::fs::create_dir_all(self.netease_dir()).is_err() {
            return;
        }
        let entry = CacheEntry {
            lrc: lrc.to_string(),
            tlyric: tlyric.to_string(),
            cached_at: Self::now_secs(),
        };
        if let Ok(content) = serde_json::to_string(&entry) {
            let _ = std::fs::write(self.cache_path(track_id), content);
        }
    }

    pub fn cleanup_expired(&self, ttl_days: u32) {
        if ttl_days == 0 {
            return;
        }
        let dir = self.netease_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else { return };
        let threshold = ttl_days as u64 * 86400;
        let now = Self::now_secs();
        let mut removed = 0usize;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                    if now.saturating_sub(cache_entry.cached_at) > threshold {
                        if std::fs::remove_file(&path).is_ok() {
                            removed += 1;
                        }
                    }
                }
            }
        }

        if removed > 0 {
            info!("Cleaned up {} expired lyric cache entries", removed);
        }
    }
}

lazy_static! {
    pub(crate) static ref LYRIC_CACHE: LyricCache = LyricCache::new();
}
