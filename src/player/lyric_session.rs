use log::info;
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
    fn drop(&mut self) {
        // stop lyric timer session if still running
        self.handle.take().map(|h| h.abort());
        // push empty lyric to clear displayed lyric
        self.msgtx.send(ChannelMessage::update_lyric_line(0, String::new(), None)).ok();
    }
}

impl SessionManager {
    #[inline]
    fn scaled_elapsed_ms(elapsed_ms: u64, rate: f64) -> u64 {
        if rate == 1.0 {
            elapsed_ms
        } else {
            ((elapsed_ms as f64) / rate) as u64
        }
    }

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
        // push empty lyric to clear displayed lyric
        self.msgtx.send(ChannelMessage::update_lyric_line(0, String::new(), None)).ok();

        // start new session
        let msgtx = self.msgtx.clone();
        let pbtx = self.pbtx.clone();
        let pbrx = self.pbtx.subscribe();
        let config = self.config.clone();
        self.handle = Some(tokio::spawn(async move {
            Self::start_lyric_session(config, lyric, msgtx, pbtx, pbrx).await;
        }));
    }

    pub(crate) async fn start_lyric_session(
        config: Arc<RwLock<Config>>,
        lyric: Lyric,
        msgtx: Sender<ChannelMessage>,
        pbtx: Sender<PlaybackEvent>,
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
                    let target_millis_after_anchor = Self::scaled_elapsed_ms(
                        target_time.saturating_sub(anchor_music_time),
                        rate,
                    );
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
                    if let Ok(ev) = &event && !matches!(ev, PlaybackEvent::Poll(_)) {
                        info!("{}", ev.to_string());
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
                        Ok(PlaybackEvent::Poll(real_music_time)) => {
                            let now = Instant::now();
                            let elapsed_ms = now.duration_since(anchor_realworld_time).as_millis() as u64;
                            let expected_music_time = anchor_music_time.saturating_add(Self::scaled_elapsed_ms(elapsed_ms, rate));
                            if real_music_time.abs_diff(expected_music_time) > 100 {
                                // Position updated!
                                // Reuse "Seek" event for simplicity, even though it's not exactly a seek
                                pbtx.send(PlaybackEvent::Seek(real_music_time)).ok();
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

    pub(crate) async fn send(&self, event: PlaybackEvent) {
        self.pbtx.send(event).ok();
    }
}
