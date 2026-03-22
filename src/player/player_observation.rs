use futures_util::stream::StreamExt;

use crate::player::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};

impl MprisListener {
    pub(crate) async fn start_observation(&self, player_id: String) -> zbus::Result<()> {
        // start player observation on the player on top of the stack
        {
            if self.config.read().await.verbose {
                println!("Starting observation for player: {}", player_id);
            }
        }

        // TODO: get properties first, then listen for changes, to avoid missing any changes between these two steps

        let service = format!("{}{}", MPRIS_PREFIX, player_id);
        let player_properties_proxy = PlayerPropertiesProxy::new(&self.conn, service).await?;
        let mut properties_changed_stream =
            player_properties_proxy.receive_properties_changed().await?;

        while let Some(signal) = properties_changed_stream.next().await {
            let args = signal.args().expect("Error parsing message");
            if self.config.read().await.verbose {
                println!(
                    "Received PropertiesChanged signal: interface={}, changed_properties={:?}, invalidated_properties={:?}",
                    args.interface_name, args.changed_properties, args.invalidated_properties
                );
            }
        }
        Ok(())
    }
}
