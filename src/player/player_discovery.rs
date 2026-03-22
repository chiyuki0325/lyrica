use futures_util::stream::StreamExt;
use tokio::sync::mpsc::Sender;

use super::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};
use crate::helpers::StringVecExt;

pub(super) enum PlayerEvent {
    Added(String),
    Removed(String),
}

impl MprisListener {
    pub(super) async fn start_discovery(&self, tx: Sender<PlayerEvent>) -> zbus::Result<()> {
        let dbus_proxy = DBusProxy::new(&self.conn).await?;
        // search for existing players
        let names = dbus_proxy.list_names().await?;
        for name in names {
            if let Some(player_id) = name.strip_prefix(MPRIS_PREFIX) {
                self.add_player(player_id, &tx).await;
            }
        }

        // listen for new players
        let mut name_owner_changed_stream = dbus_proxy.receive_name_owner_changed().await?;

        while let Some(signal) = name_owner_changed_stream.next().await {
            let args = signal.args().expect("Error parsing message");

            if let Some(player_id) = args.name.strip_prefix(MPRIS_PREFIX) {
                // mpv or mpv.instance-PdGSpHZs

                let is_new_player = !args.new_owner.is_empty();

                if is_new_player {
                    self.add_player(player_id, &tx).await;
                } else {
                    self.remove_player(player_id, &tx).await;
                }
            }
        }
        Ok(())
    }

    async fn add_player(&self, player_id: &str, tx: &Sender<PlayerEvent>) {
        let player_sort = player_id.split('.').next().unwrap_or(player_id);
        // "mpv" itself

        if !self
            .config
            .read()
            .await
            .disabled_players
            .contains(player_sort)
        {
            tx.send(PlayerEvent::Added(player_id.to_string()))
                .await
                .ok();
            if self.config.read().await.verbose {
                println!("Player added: {}", player_id);
            }
        }
    }

    async fn remove_player(&self, player_id: &str, tx: &Sender<PlayerEvent>) {
        tx.send(PlayerEvent::Removed(player_id.to_string()))
            .await
            .ok();
        if self.config.read().await.verbose {
            println!("Player removed: {}", player_id);
        }
    }
}
