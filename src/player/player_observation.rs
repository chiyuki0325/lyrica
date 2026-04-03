use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use futures_util::stream::StreamExt;
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;
use std::fmt;
use tokio::sync::broadcast::{Sender, channel};
use zbus::zvariant::OwnedValue;

use crate::lyric_providers::try_get_lyric_from_providers;
use crate::messages::ChannelMessage;
use crate::player::lyric_session::SessionManager;
use crate::player::mpris_metadata::Metadata;
use crate::player::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};

lazy_static! {
    // Players that not emitting seeked signal
    // So we have to keep polling position to detect seek
    pub(crate) static ref POLL_PLAYERS: Vec<String> = vec![
        "elisa".to_string(),
    ];
}

#[derive(Debug, Clone)]
pub(crate) enum PlaybackEvent {
    Play,
    Pause,
    Seek(u64),
    RateChange(f64),
    Poll(u64), // only for polling mode, not emitted by players
    Reset(),   // song changed
}

impl fmt::Display for PlaybackEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybackEvent::Play => write!(f, "- Play"),
            PlaybackEvent::Pause => write!(f, "- Pause"),
            PlaybackEvent::Seek(pos) => write!(f, "- Seek({} ms)", pos),
            PlaybackEvent::RateChange(rate) => write!(f, "- RateChange({})", rate),
            PlaybackEvent::Poll(position) => write!(f, "- Poll({} ms)", position),
            PlaybackEvent::Reset() => write!(f, "- Reset()"),
        }
    }
}

impl MprisListener {
    async fn player_proxy<'a>(&self, service_name: &'a str) -> zbus::Result<PlayerProxy<'a>> {
        PlayerProxy::builder(&self.conn)
            .destination(service_name)?
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await
    }

    async fn player_properties_proxy<'a>(
        &self,
        service_name: &'a str,
    ) -> zbus::Result<PlayerPropertiesProxy<'a>> {
        PlayerPropertiesProxy::builder(&self.conn)
            .destination(service_name)?
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await
    }

    pub(crate) async fn start_observation<'a>(&self, player_id: String) -> zbus::Result<()> {
        let service_name = format!("{}{}", MPRIS_PREFIX, player_id);
        let player_proxy = self.player_proxy(&service_name).await?;
        let player_properties_proxy = self.player_properties_proxy(&service_name).await?;

        // channel for playback events, received in lyric session
        let (pbtx, pbrx) = channel::<PlaybackEvent>(16);
        drop(pbrx); // sub in lyric session layer
        let mut ssmgr: SessionManager = SessionManager::new(
            player_id.clone(),
            self.config.clone(),
            self.tx.clone(),
            pbtx.clone(),
        )
        .await;

        // try spawn lyric session if song is already playing
        self.bootstrap_lyric_session(&player_proxy, &mut ssmgr)
            .await;

        // start listening for property changes
        let pbtx1 = pbtx.clone();
        let listen_properties_handle =
            self.start_listen_properties(&player_proxy, &service_name, ssmgr, pbtx1);

        // start listening for seeked signal
        let pbtx2 = pbtx.clone();
        let listen_seeked_handle: BoxFuture<'_, zbus::Result<()>> = if POLL_PLAYERS
            .iter()
            .any(|p| player_id.starts_with(p))
        {
            // start polling position if player is in polling mode list
            info!(
                "Entering polling mode because of the weird behavior of player {}, high latency expected",
                player_id
            );
            self.start_poll_position(&player_proxy, pbtx2).boxed()
        } else {
            self.start_listen_seeked(&player_proxy, pbtx2).boxed()
        };

        drop(pbtx);

        tokio::try_join!(listen_properties_handle, listen_seeked_handle)
            .map(|_| ())
            .map_err(|_| zbus::Error::Failure(String::from("Player exited")))
    }

    async fn start_listen_properties<'a>(
        &self,
        player_proxy: &PlayerProxy<'a>,
        service_name: &str,
        mut ssmgr: SessionManager,
        pbtx: Sender<PlaybackEvent>,
    ) -> zbus::Result<()> {
        let player_properties_proxy = self.player_properties_proxy(service_name).await?;
        let mut properties_stream = player_properties_proxy.receive_properties_changed().await?;

        while let Some(signal) = properties_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            for event in self
                .dispatch_properties(player_proxy, args.changed_properties)
                .await
                .into_iter()
            {
                match event {
                    PlaybackEvent::Reset() => {
                        // if song changed, restart lyric session
                        self.bootstrap_lyric_session(player_proxy, &mut ssmgr).await;
                    }
                    _ => {
                        pbtx.send(event).ok();
                    }
                }
            }
        }

        Ok(())
    }

    async fn start_listen_seeked<'a>(
        &self,
        player_proxy: &PlayerProxy<'a>,
        pbtx: Sender<PlaybackEvent>,
    ) -> zbus::Result<()> {
        let mut seeked_stream = player_proxy.receive_seeked().await?;

        while let Some(signal) = seeked_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            pbtx.send(PlaybackEvent::Seek((args.position / 1000) as u64))
                .ok();
        }

        Ok(())
    }

    async fn start_poll_position<'a>(
        &self,
        player_proxy: &PlayerProxy<'a>,
        pbtx: Sender<PlaybackEvent>,
    ) -> zbus::Result<()> {
        let mut position = 0;
        loop {
            let new_position = player_proxy.position().await.ok().unwrap_or(0);
            if new_position != position {
                // manually emit seeked signal
                pbtx.send(PlaybackEvent::Poll((new_position / 1000) as u64))
                    .ok();
                position = new_position;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn bootstrap_lyric_session<'a>(
        &self,
        player_proxy: &PlayerProxy<'a>,
        ssmgr: &mut SessionManager,
    ) -> Option<()> {
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
            .send(ChannelMessage::update_music_info(title, artist))
            .ok();
    }

    async fn dispatch_properties<'a>(
        &self,
        player_proxy: &PlayerProxy<'a>,
        changed_properties: HashMap<String, OwnedValue>,
    ) -> Vec<PlaybackEvent> {
        let mut events = Vec::new();

        for (key, value) in changed_properties.into_iter() {
            // info!("Property changed: {} = {:?}", key, value);
            match key.as_str() {
                "Metadata" => {
                    events.push(PlaybackEvent::Reset());
                    // immediately return after song change
                    return events;
                }
                "Rate" => {
                    if let Ok(rate) = value.downcast_ref::<f64>() {
                        events.push(PlaybackEvent::RateChange(rate));

                        // push latest time when rate changed to trigger re-anchor
                        let position = player_proxy.position().await.ok().unwrap_or(0);
                        events.push(PlaybackEvent::Seek((position / 1000) as u64));
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
                        // push latest time when pause/play
                        let position = player_proxy.position().await.ok().unwrap_or(0);
                        events.push(PlaybackEvent::Seek((position / 1000) as u64));
                    }
                }
                _ => continue,
            }
        }

        events
    }
}
