use std::sync::Arc;
use tokio::{sync::mpsc::channel, task::JoinHandle};

use super::{MprisListener, player_discovery::PlayerEvent};
use crate::helpers::StringVecExt;

impl MprisListener {
    pub(super) async fn start_mpris_loop(self) -> zbus::Result<()> {
        // start player discovery
        let this = Arc::new(self);

        let (tx, mut rx) = channel::<PlayerEvent>(4);

        let self_for_discovery = this.clone();
        tokio::spawn(async move { self_for_discovery.start_discovery(tx).await });

        let mut players = Vec::<String>::new();
        let mut handle: Option<JoinHandle<()>> = Option::<JoinHandle<()>>::None;

        while let Some(event) = rx.recv().await {
            match event {
                PlayerEvent::Added(player_id) => {
                    players.push(player_id);
                }
                PlayerEvent::Removed(player_id) => {
                    if let Some(pos) = players.position_of(&player_id) {
                        players.remove(pos);
                    }
                }
            }

            // restart observation if the player on top of the stack changes
            let player_on_top = players.last().cloned();

            // kill existing observation task
            handle.take().map(|h| h.abort());

            if let Some(player_id) = player_on_top {
                let self_for_observation = this.clone();
                handle = Some(tokio::spawn(async move {
                    if let Err(e) = self_for_observation.start_observation(player_id).await {
                        eprintln!("Error in MPRIS observation: {}", e);
                    }
                }));
            }
        }

        Ok(())
    }
}
