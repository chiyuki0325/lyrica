use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Sender, channel};
use zbus::zvariant::OwnedValue;

use crate::lyric_providers::try_get_lyric_from_providers;
use crate::messages::{ChannelMessage, UpdateMusicInfoData};
use crate::player::lyric_session::SessionManager;
use crate::player::mpris_metadata::Metadata;
use crate::player::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};

#[derive(Debug, Clone)]
pub(crate) enum PlaybackEvent {
    Play,
    Pause,
    Seek(u64),
    RateChange(f64),
    Reset(), // song changed
}

impl MprisListener {
    pub(crate) async fn start_observation(self: Arc<Self>, player_id: String) -> zbus::Result<()> {
        let service_name: Arc<str> = format!("{}{}", MPRIS_PREFIX, player_id).into();

        // channel for playback events, received in lyric session
        let (pbtx, pbrx) = channel::<PlaybackEvent>(16);
        let mut ssmgr: SessionManager =
            SessionManager::new(player_id.clone(), self.config.clone(), pbtx.clone()).await;

        // try spawn lyric session if song is already playing
        self.clone()
            .bootstrap_lyric_session(&service_name, &mut ssmgr)
            .await;

        // start listening for property changes
        let listen_properties_handle = {
            let this = self.clone();
            let service_name = service_name.clone();
            let pbtx1 = pbtx.clone();
            tokio::spawn(async move {
                this.start_listen_properties(&service_name, ssmgr, pbtx1)
                    .await
                    .ok();
            })
        };

        // start listening for seeked signal
        let listen_seeked_handle = {
            let this = self.clone();
            let service_name = service_name.clone();
            let pbtx1 = pbtx.clone();
            tokio::spawn(async move {
                this.start_listen_seeked(&service_name, pbtx1).await.ok();
            })
        };

        tokio::try_join!(listen_properties_handle, listen_seeked_handle)
            .map(|_| ())
            .map_err(|_| zbus::Error::Failure(String::from("Player exited")))
    }

    async fn start_listen_properties(
        self: Arc<Self>,
        service_name: &str,
        mut ssmgr: SessionManager,
        pbtx: Sender<PlaybackEvent>,
    ) -> zbus::Result<()> {
        let player_properties_proxy = PlayerPropertiesProxy::new(&self.conn, service_name).await?;
        let mut properties_stream = player_properties_proxy.receive_properties_changed().await?;

        while let Some(signal) = properties_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            for event in self
                .dispatch_properties(args.changed_properties)
                .await
                .into_iter()
            {
                match event {
                    PlaybackEvent::Reset() => {
                        // if song changed, restart lyric session
                        self.clone()
                            .bootstrap_lyric_session(service_name, &mut ssmgr)
                            .await;
                    }
                    _ => {
                        pbtx.send(event).ok();
                    }
                }
            }
        }

        Ok(())
    }

    async fn start_listen_seeked(
        self: Arc<Self>,
        service_name: &str,
        pbtx: Sender<PlaybackEvent>,
    ) -> zbus::Result<()> {
        let player_proxy = PlayerProxy::new(&self.conn, service_name).await?;
        let mut seeked_stream = player_proxy.receive_seeked().await?;

        while let Some(signal) = seeked_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            pbtx.send(PlaybackEvent::Seek((args.position / 1000) as u64))
                .ok();
        }

        Ok(())
    }

    async fn bootstrap_lyric_session(
        self: Arc<Self>,
        service_name: &str,
        ssmgr: &mut SessionManager,
    ) -> Option<()> {
        let player_proxy = PlayerProxy::new(&self.conn, service_name).await.ok()?;
        let metadata = player_proxy.metadata().await.ok()?;
        let playback_status = player_proxy.playback_status().await.ok()?;
        let rate = player_proxy.rate().await.ok().unwrap_or(1.0);
        let position = player_proxy.position().await.ok().unwrap_or(0);

        if metadata.has_song_info() {
            // song playing when we start observation, start lyric session immediately
            let lyric = try_get_lyric_from_providers(&metadata, self.config.clone()).await;

            if let Some(lyric) = lyric {
                ssmgr.start_session(lyric).await;

                // push music info to UI
                self.push_music_info(metadata).await;

                // push playback status to lyric session
                let _ = ssmgr.send(PlaybackEvent::Seek((position / 1000) as u64));
                let _ = ssmgr.send(PlaybackEvent::RateChange(rate));
                let _ = ssmgr.send(match playback_status {
                    PlaybackStatus::Playing => PlaybackEvent::Play,
                    PlaybackStatus::Paused => PlaybackEvent::Pause,
                    PlaybackStatus::Stopped => PlaybackEvent::Pause,
                });

                ()
            }
        }

        None
    }

    async fn push_music_info(&self, metadata: Metadata) {
        let title = metadata.title().unwrap_or_default();
        let artist = metadata.artist().unwrap_or_default();

        self.tx
            .send(ChannelMessage::UpdateMusicInfo(UpdateMusicInfoData {
                title,
                artist,
            }))
            .ok();
    }

    async fn dispatch_properties(
        &self,
        changed_properties: HashMap<String, OwnedValue>,
    ) -> Vec<PlaybackEvent> {
        let mut events = Vec::new();

        for (key, value) in changed_properties.into_iter() {
            // println!("Property changed: {} = {:?}", key, value);
            match key.as_str() {
                "Metadata" => {
                    events.push(PlaybackEvent::Reset());
                    // immediately return after song change
                    return events;
                }
                "Rate" => {
                    if let Ok(rate) = value.downcast_ref::<f64>() {
                        events.push(PlaybackEvent::RateChange(rate));
                    }
                }
                "PlaybackStatus" => {
                    if let Ok(status) = value.downcast_ref::<String>() {
                        events.push(match status.as_str() {
                            "Playing" => PlaybackEvent::Play,
                            "Paused" => PlaybackEvent::Pause,
                            "Stopped" => PlaybackEvent::Pause,
                            _ => continue,
                        });
                    }
                }
                _ => continue,
            }
        }

        events
    }
}
