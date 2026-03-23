use std::sync::Arc;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::lyric_parser::Lyric;
use crate::player::mpris_metadata::Metadata;

enum LyricProviderError {
    NotFound,
    Aborted,
}

trait LyricProvider {
    fn get_lyric(&self, metadata: &Metadata, config: Arc<RwLock<Config>>) -> Result<Lyric, LyricProviderError>;
    fn is_available(&self, metadata: &Metadata, config: Arc<RwLock<Config>>) -> bool;
}

type LyricProviderList = Vec<Box<dyn LyricProvider + Send + Sync>>;

lazy_static! {
    static ref LYRIC_PROVIDERS: LyricProviderList = {
        let providers: LyricProviderList = vec![
        ];
        providers
    };
}