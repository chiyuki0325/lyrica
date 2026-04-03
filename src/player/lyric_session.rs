use std::future::pending;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio::time::sleep_until;

use crate::config::Config;
use crate::messages::ChannelMessage;
use crate::player::MPRIS_PREFIX;
use crate::{lyric_parser::Lyric, player::player_observation::PlaybackEvent};

pub(crate) struct SessionManager {
    player_id: String,
    service_name: String,
    config: Arc<RwLock<Config>>,
    handle: Option<JoinHandle<()>>,
    msgtx: Sender<ChannelMessage>,
    pbtx: Sender<PlaybackEvent>,
}

impl Drop for SessionManager {
    // Detect leak, stops leak
    fn drop(&mut self) {
        self.handle.take().map(|h| h.abort());
    }
}

impl SessionManager {
    pub async fn new(
        player_id: String,
        config: Arc<RwLock<Config>>,
        msgtx: Sender<ChannelMessage>,
        pbtx: Sender<PlaybackEvent>,
    ) -> Self {
        Self {
            player_id: player_id.clone(),
            service_name: format!("{}{}", MPRIS_PREFIX, player_id),
            config: config,
            handle: None,
            msgtx: msgtx,
            pbtx: pbtx,
        }
    }
    pub async fn start_session(&mut self, lyric: Lyric) {
        // terminate existing session if exists
        self.handle.take().map(|h| h.abort());

        // start new session
        let msgtx = self.msgtx.clone();
        let pbrx = self.pbtx.subscribe();
        let config = self.config.clone();
        self.handle = Some(tokio::spawn(async move {
            Self::start_lyric_session(config, lyric, msgtx, pbrx).await;
        }));
    }

    pub(crate) async fn start_lyric_session(
        config: Arc<RwLock<Config>>,
        lyric: Lyric,
        msgtx: Sender<ChannelMessage>,
        mut pbrx: Receiver<PlaybackEvent>,
    ) {
        let mut anchor_realworld_time = Instant::now();
        let mut anchor_music_time = 0;

        let lyric_lines = lyric.lines.len();
        let mut last_position = usize::max_value();
        let mut position = 0;
        let mut rate = 1.0;
        let mut paused = false;

        if lyric_lines == 0 {
            return;
        }

        loop {
            let timer = async {
                if !paused && position < lyric_lines {
                    let target_time = lyric.lines[position].time;
                    let target_millis_after_anchor =
                        ((target_time - anchor_music_time) as f64 * rate) as u64;
                    let target_instant =
                        anchor_realworld_time + Duration::from_millis(target_millis_after_anchor);
                    sleep_until(target_instant).await;
                } else {
                    pending::<()>().await;
                }
            };

            tokio::select! {
                _ = timer => {
                    if position < lyric_lines {
                            let current_line = &lyric.lines[position];

                            msgtx.send(ChannelMessage::update_lyric_line(
                                current_line.time,
                                current_line.text.clone(),
                                current_line.alt.clone(),
                            )).ok();

                            position += 1;
                    }
                }

                event = pbrx.recv() => {
                    if config.read().await.verbose {
                        println!("{}", event.as_ref().map(|e| e.to_string()).unwrap_or_default());
                    }
                    match event {
                        Ok(PlaybackEvent::Seek(new_time)) => {
                            // Adjust anchor time based on new position

                            anchor_music_time = new_time;
                            anchor_realworld_time = Instant::now();

                            position = lyric.lines.iter().position(|line| line.time > new_time).unwrap_or(lyric_lines);

                            // fix: before first line
                            if position != last_position {
                                last_position = position;
                                if position == 0 {
                                    msgtx.send(ChannelMessage::update_lyric_line(
                                        0,
                                        String::new(),
                                        None,
                                    )).ok();
                                } else {
                                    let line = &lyric.lines[position.saturating_sub(1)];
                                    msgtx.send(ChannelMessage::update_lyric_line(
                                        line.time,
                                        line.text.clone(),
                                        line.alt.clone(),
                                    )).ok();
                                }
                            }
                        }
                        Ok(PlaybackEvent::Pause) => {
                            paused = true;
                        }
                        Ok(PlaybackEvent::Play) => {
                            paused = false;
                            // a Seeked event will be triggered immediately after Play
                            // so we don't need to adjust position here
                        }
                        Ok(PlaybackEvent::RateChange(new_rate)) => {
                            rate = new_rate;
                        }
                        Ok(PlaybackEvent::Reset()) => {
                            // lyric session will be quit, so do nothing here
                            break;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn send(&self, event: PlaybackEvent) {
        self.pbtx.send(event).ok();
    }
}
