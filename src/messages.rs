use std::fmt::Display;

use crate::config::Config;
use serde::Serialize;
use serde;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChannelMessage {
    UpdateLyricLine(UpdateLyricLineData),
    UpdateMusicInfo(UpdateMusicInfoData),
    UpdateConfig(Config),
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateLyricLineData {
    pub time: u128,
    pub lyric: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateMusicInfoData {
    pub title: String,
    pub artist: String,
}

impl Display for ChannelMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelMessage::UpdateLyricLine(data) => write!(f, "-> [{}] {}", data.time, data.lyric),
            ChannelMessage::UpdateMusicInfo(data) => write!(f, "-> [{} - {}]", data.title, data.artist),
            ChannelMessage::UpdateConfig(_) => write!(f, "-> [Config Update]"),
        }
    }
}