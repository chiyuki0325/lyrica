use std::fmt::Display;

use crate::config::Config;
use serde;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChannelMessage {
    UpdateLyricLine(UpdateLyricLineData),
    UpdateMusicInfo(UpdateMusicInfoData),
    UpdateConfig(Config),
}

impl ChannelMessage {
    pub(crate) fn update_lyric_line(time: u64, text: String, alt: Option<String>) -> Self {
        ChannelMessage::UpdateLyricLine(UpdateLyricLineData { time, text, alt })
    }

    pub(crate) fn update_music_info(title: String, artist: String) -> Self {
        ChannelMessage::UpdateMusicInfo(UpdateMusicInfoData { title, artist })
    }

    pub(crate) fn update_config(config: Config) -> Self {
        ChannelMessage::UpdateConfig(config)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateLyricLineData {
    pub time: u64,
    pub text: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateMusicInfoData {
    pub title: String,
    pub artist: String,
}

impl Display for ChannelMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelMessage::UpdateLyricLine(data) => write!(
                f,
                "-> [{}] {} {}",
                data.time,
                data.text,
                data.alt.as_deref().unwrap_or("")
            ),
            ChannelMessage::UpdateMusicInfo(data) => {
                write!(f, "-> [{} - {}]", data.title, data.artist)
            }
            ChannelMessage::UpdateConfig(_) => write!(f, "-> [Config Update]"),
        }
    }
}
