use futures_util::stream::StreamExt;
use std::collections::HashMap;
use zbus::zvariant::{Dict, OwnedValue};

use crate::messages::{ChannelMessage, UpdateMusicInfoData};
use crate::player::mpris_metadata::Metadata;
use crate::player::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};

impl MprisListener {
    pub(crate) async fn start_observation(&self, player_id: String) -> zbus::Result<()> {
        // start player observation on the player on top of the stack
        {
            if self.config.read().await.verbose {
                println!("Starting observation for player: {}", player_id);
            }
        }

        // get properties first, then listen for changes
        let service = format!("{}{}", MPRIS_PREFIX, player_id);

        let player_proxy = PlayerProxy::new(&self.conn, service.clone()).await?;
        let metadata = player_proxy.metadata().await?;

        let title = metadata.title().unwrap_or_default();
        let artist = metadata.artist().unwrap_or_default();

        if !(title.is_empty() && artist.is_empty()) {
            self.tx
                .send(ChannelMessage::UpdateMusicInfo(UpdateMusicInfoData {
                    title,
                    artist,
                }))
                .ok();
        }

        // start listening for property changes
        let player_properties_proxy = PlayerPropertiesProxy::new(&self.conn, service).await?;
        let mut properties_changed_stream =
            player_properties_proxy.receive_properties_changed().await?;

        while let Some(signal) = properties_changed_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            self.dispatch_properties_changed(&args.interface_name, args.changed_properties)
                .await;
        }
        Ok(())
    }

    async fn dispatch_properties_changed(
        &self,
        interface_name: &str,
        changed_properties: HashMap<String, OwnedValue>,
    ) {
        if self.config.read().await.verbose {
            println!(
                "Properties changed for interface {}: {:?}",
                interface_name, changed_properties
            );
        }

        for (key, value) in changed_properties.into_iter() {
            match key.as_str() {
                "Metadata" => {
                    if let Ok(metadata) = Metadata::try_from(value) {
                        let title = metadata.title().unwrap_or_default();
                        let artist = metadata.artist().unwrap_or_default();

                        self.tx
                            .send(ChannelMessage::UpdateMusicInfo(UpdateMusicInfoData {
                                title,
                                artist,
                            }))
                            .ok();
                    }
                }
                _ => (),
            }
        }
    }
}
