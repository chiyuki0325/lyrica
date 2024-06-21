// https://github.com/Techno3d/mpris-rs-async
// Removed panic for this project to use

//! [`MyPlayerStream`] Handles player connections. It will try for a connection to a new mpris player forever.

use std::{time::Duration, task::{Waker, Poll}};

use async_std::{task, stream::Stream};
use mpris::{Player, PlayerFinder};

/// The MyPlayerStream, which will keep checking for players forever. Created by calling [`crate::stream_players`]
#[derive(Default, Debug)]
pub struct MyPlayerStream {
    players: Vec<Player>,
    index: usize,
    retry_delay: u64,
}

impl MyPlayerStream {
    /// Creates a new [`MyPlayerStream`]. Every `retry_delay` milliseconds it will try for a new
    /// connection.
    pub fn new(retry_delay: u64) -> Self {
        return MyPlayerStream { players: vec![], index: 0, retry_delay };
    }

    async fn wake_after_change(waker: Waker, retry_delay: u64, players_len: usize) {
        loop {
            task::sleep(Duration::from_millis(retry_delay)).await;
            let finder = match PlayerFinder::new() {
                Ok(x) => x,
                Err(e) => panic!("Unexpected DBus Error: {}", e),
            };
            let found_players = match finder.find_all() {
                Ok(x) => x,
                Err(mpris::FindingError::NoPlayerFound) => continue,
                Err(mpris::FindingError::DBusError(_e)) => {
                    //eprintln!("DbusError: {}", e);
                    continue;
                },
            };
            if found_players.len() != players_len {
                waker.wake_by_ref();
                return;
            }
        }
    }
}

impl Stream for MyPlayerStream {
    type Item = Player;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Option<Self::Item>> {
        let finder = match PlayerFinder::new() {
            Ok(x) => x,
            //Err(e) => panic!("DBus Error: {}", e),
            Err(_) => return Poll::Ready(None),
        };
        if self.players.len() != 0 {
            let mut decrement_num = 0;
            // Filters out dead connections
                self.players = self.players.iter().filter(|x| {
                    if !x.is_running() {
                        decrement_num += 1;
                    }
                    x.is_running()
                }).filter_map(|x| {
                    match finder.find_by_name(x.identity()) {
                        Ok(x) => Some(x),
                        // Err(e) => panic!("Unexpected error. Player may have quit after check. {}", e),
                        // DON' T PANIC.
                        Err(_) => None,
                    }
                }).collect();

            self.index -= decrement_num;

            let all_players = match finder.find_all() {
                Ok(x) => x,
                Err(_) => return Poll::Ready(None),
                //Err(e) => panic!("DBus error? {}", e),
            };

            //Kind of goofy work around
            let new_players = all_players.iter().filter(|x| {
                let copy_of_players = self.players.iter();
                for potential_player in copy_of_players {
                    if potential_player.unique_name() == x.unique_name() {
                        return false;
                    }
                }
                return true;
            }).map(|x| {finder.find_by_name(x.identity()).unwrap()}).collect::<Vec<Player>>();

            self.players.extend(new_players);
        } else {
            let all_players = match finder.find_all() {
                Ok(x) => x,
                //Err(_) => return Poll::Ready(None),
                Err(e) => panic!("DBus error? {}", e),
            };
            self.players.extend(all_players);
        }

        // Basically manually making an iterator
        if self.index < self.players.len() {
            let new_player: Player = finder.find_by_name(self.players.get(self.index).unwrap().identity()).unwrap();
            self.index += 1;
            return Poll::Ready(Some(new_player));
        } else {
            let waker = cx.waker().to_owned();
            let retry_delay = self.retry_delay;
            let players_len = self.players.len();
            task::spawn(async move {
                MyPlayerStream::wake_after_change(waker, retry_delay, players_len).await;
            });
            return Poll::Pending;
        }
    }
}

