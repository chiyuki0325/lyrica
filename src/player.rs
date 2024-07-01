use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use crate::{ChannelMessage, lyric_providers};
use crate::config::SharedConfig;
use crate::player_stream::MyPlayerStream;
use async_std::stream::StreamExt;
use std::sync::{Arc, Mutex};
use lyric_providers::LyricProvider;
use crate::lyric_parser::{
    parse_lyrics,
    LyricLine,
};

struct MprisInfo {
    url: String,
    is_lyric: bool,
    player_running: bool,
}

type MprisCache = Arc<Mutex<MprisInfo>>;

pub async fn mpris_loop(
    tx: broadcast::Sender<ChannelMessage>,
    config: SharedConfig,
) {
    let mut cache = Arc::new(Mutex::new((MprisInfo {
        url: String::new(),
        is_lyric: false,
        player_running: false,
    })));

    let mut player_stream = MyPlayerStream::new(500);
    while let Some(player) = player_stream.next().await {
        // 得到播放器，进入循环
        if config.read().unwrap().verbose {
            println!("New player connected: {:?}", player.bus_name());
        }
        let player_name = String::from(player.bus_name());
        let player_name = player_name.strip_prefix("org.mpris.MediaPlayer2.").unwrap();
        if config.read().unwrap().disabled_players.contains(&player_name.to_string()) {
            if config.read().unwrap().verbose {
                println!("Player {} detected, but disabled in the config.", player_name);
            }
            continue;
        }

        cache.lock().unwrap().player_running = true;

        let mut idx = 0;
        let mut last_time: u128 = 0;
        // 这个变量是循环上一次运行时的时间，用于判断进度条是否往左拉了
        let mut lyric: Vec<LyricLine> = Vec::new();
        let tlyric_mode = config.read().unwrap().tlyric_mode;

        loop {
            // 主循环，此时 player 已被移动到此大括号中
            match player.get_metadata() {
                Ok(metadata) => {
                    // 判断歌曲是否更改
                    let url = metadata.url().unwrap_or(
                        metadata.art_url().unwrap_or_default()
                    );

                    let mut cache = cache.lock().unwrap();


                    if cache.url != url {
                        // 歌曲更改
                        if config.read().unwrap().verbose {
                            println!("New song detected: {}", url);
                        }
                        cache.url = url.to_string();

                        tx.send(ChannelMessage::UpdateMusicInfo(
                            metadata.title().unwrap_or_default().to_string(),
                            metadata.artists().unwrap_or_default()[0].to_string(),
                        )).unwrap();

                        // 尝试获取歌词
                        lyric = Vec::new();
                        cache.is_lyric = false;
                        for (name, provider) in lyric_providers::LYRIC_PROVIDERS.iter() {
                            if config.read().unwrap().enabled_lyric_providers.contains(&(name.to_string())) {
                                if config.read().unwrap().verbose {
                                    println!("Trying provider: {}", name);
                                }
                                // 这个 provider 可用
                                if provider.is_available(&url) {
                                    // 这个 provider 可以处理这个 URL
                                    let mut success = false;
                                    (lyric, success) = provider.get_lyric(&url, &metadata).await;
                                    if success {
                                        // 成功获取歌词
                                        if config.read().unwrap().verbose {
                                            println!("Got lyric from provider: {}", name);
                                        }
                                        // 解析歌词并且存入 lyric
                                        cache.is_lyric = true;
                                        idx = 0;
                                        last_time = 0;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // 播放器已经关闭。
                    if config.read().unwrap().verbose {
                        println!("Player closed, exiting loop...");
                    }
                    tx.send(ChannelMessage::UpdateMusicInfo("".to_string(), "".to_string())).unwrap();
                    // 跳出 loop 块，继续等待下一个播放器
                    cache.lock().unwrap().player_running = false;
                    break;
                }
            }
            // 歌词是否变化？

            // 获取当前时间
            if cache.lock().unwrap().is_lyric {
                let current_time = player.get_position().unwrap_or_default().as_micros();

                if current_time < last_time {
                    // 进度条往左拉了，重置 idx
                    idx = 0;
                    while idx < lyric.len() && current_time >= lyric[idx].time {
                        idx += 1;
                    }
                }

                let line = lyric.get(idx);
                if let Some(line) = line {
                    if current_time >= line.time {
                        // 歌词变化
                        let line_lyric = if line.tlyric.is_some() {
                            match tlyric_mode {
                                1 => line.tlyric.clone().unwrap(),
                                2 => format!("{} | {}", line.lyric, line.tlyric.clone().unwrap()),
                                3 => format!("{} | {}", line.tlyric.clone().unwrap(), line.lyric),
                                _ => line.lyric.clone(),
                            }
                        } else {
                            line.lyric.clone()
                        };

                        tx.send(ChannelMessage::UpdateLyricLine(line.time, line_lyric)).unwrap();
                        while idx < lyric.len() && current_time >= lyric[idx].time {
                            idx += 1;
                        }
                    }
                }

                last_time = current_time;
            }

            sleep(Duration::from_millis(50)).await;
        }
    }
}
