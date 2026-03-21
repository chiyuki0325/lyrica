use futures_util::stream::StreamExt;

use crate::helpers::StringVecExt;
use crate::player::{MPRIS_PREFIX, MprisListener, dbus_proxies::*};

impl MprisListener {
    pub(crate) async fn start_discovery(&mut self) -> zbus::Result<()> {
        println!("Starting MPRIS player discovery...");

        let dbus_proxy = DBusProxy::new(&self.conn).await?;
        let mut name_owner_changed_stream = dbus_proxy.receive_name_owner_changed().await?;

        while let Some(signal) = name_owner_changed_stream.next().await {
            let args = signal.args().expect("Error parsing message");

            if let Some(player_id) = args.name.strip_prefix(MPRIS_PREFIX) {
                // mpv or mpv.instance-PdGSpHZs

                let is_new_player = !args.new_owner.is_empty();
                if is_new_player {
                    let guard = self.config.read().await;
                    let player_sort = player_id.split('.').next().unwrap_or(player_id);
                    // "mpv" itself

                    if !guard.disabled_players.contains(player_sort) {
                        if guard.verbose {
                            println!("Player added: {}", player_id);
                        }
                        self.players.push(player_id.to_string());
                    }
                } else {
                    if let Some(pos) = self.players.position_of(player_id) {
                        self.players.remove(pos);
                        if self.config.read().await.verbose {
                            println!("Player removed: {}", player_id);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
