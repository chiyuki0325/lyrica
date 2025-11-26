use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use crate::{ChannelMessage, lyric_providers};
use crate::config::SharedConfig;
use crate::lyric_parser::{
    LyricLine,
};
use crate::timer::MprisTimer;

struct MprisInfo {
    url: String,
    is_lyric: bool,
    player_running: bool,
}

// type MprisCache = Arc<Mutex<MprisInfo>>;

pub async fn mpris_loop(
    tx: broadcast::Sender<ChannelMessage>,
    config: SharedConfig,
) {
    let mut cache = MprisInfo {
        url: String::new(),
        is_lyric: false,
        player_running: false,
    };
    let player_finder = mpris::PlayerFinder::new().expect("Failed to create player finder");

    loop {
        for player in player_finder.iter_players().expect("Failed to connect to D-Bus") {

            if player.is_err() {
                // 播放器已经关闭（此代码块可能会被执行多次，此时 player 一直是 DBusError）
                if cache.player_running {
                    if config.read().await.verbose {
                        println!("Player closed, exiting loop...");
                    }
                    tx.send(ChannelMessage::UpdateMusicInfo("".to_string(), "".to_string())).unwrap();
                    // 跳出 loop 块，继续等待下一个播放器
                    cache.player_running = false;
                }
                continue;
            }

            let player = player.unwrap();

            // 得到播放器，进入循环
            if config.read().await.verbose {
                println!("New player connected: {:?}", player.bus_name());
            }
            let player_name = String::from(player.bus_name());
            let player_name = player_name.strip_prefix("org.mpris.MediaPlayer2.").unwrap();

            let mut is_disabled = false;
            for disabled_player in config.read().await.disabled_players.iter() {
                if player_name.starts_with(disabled_player) {
                    if config.read().await.verbose {
                        println!("Player {} detected, but disabled in the config.", player_name);
                    }
                    is_disabled = true;
                    break;
                }
            }
            if is_disabled {
                continue;
            }

            cache.player_running = true;

            let mut idx = 0;
            let mut lyric: Vec<LyricLine> = Vec::new();
            let mut tlyric_mode;
            let mut timer = MprisTimer::new();

            loop {
                // 主循环，此时 player 已被移动到此大括号中
                match player.get_metadata() {
                    Ok(metadata) => {
                        // 更新设置
                        tlyric_mode = config.read().await.tlyric_mode;

                        // 判断歌曲是否更改

                        let url = if let Some(_url) = metadata.url() {
                            _url.to_string()
                        } else {
                            let art_url = metadata.art_url().unwrap_or_default().to_string();
                            if let Some(_track_id) = metadata.track_id() {
                                let track_id = _track_id.to_string();
                                if track_id.contains("org/mpris") || track_id.contains("MediaPlayer2") {
                                    // KDE Connect quirk
                                    art_url
                                } else {
                                    track_id
                                }
                            } else {
                                art_url
                            }
                        };

                        if cache.url != url {
                            // 歌曲更改
                            if config.read().await.verbose {
                                println!("New song detected: {}", url);
                            }
                            cache.url = url.clone();


                            let title = metadata.title().unwrap_or_default().to_string();
                            let artist = metadata.artists().unwrap_or_default().get(0).unwrap_or(&"").to_string();
                            // 这个 mpris 库可能抓不到歌手，需要额外做处理
                            tx.send(ChannelMessage::UpdateMusicInfo(
                                title.clone(),
                                artist,
                            )).unwrap();

                            if title.is_empty() {
                                // 没有歌曲名，不尝试获取歌词
                                cache.is_lyric = false;
                                continue;
                            }

                            // 尝试获取歌词
                            lyric = Vec::new();
                            cache.is_lyric = false;
                            for provider_id in config.read().await.enabled_lyric_providers.iter() {
                                if let Some(provider) = lyric_providers::LYRIC_PROVIDERS.get(provider_id.clone()) {

                                    if config.read().await.verbose {
                                        println!("Trying provider: {}", provider.get_name());
                                    }
                                    // 由于现在使用 provider ID，因此注释掉

                                    // 这个 provider 可用
                                    if provider.is_available(&url, &metadata) {
                                        // 这个 provider 可以处理这个 URL
                                        let success;
                                        let try_next;
                                        (lyric, success, try_next) = provider.get_lyric(
                                            &url,
                                            &metadata,
                                            config.clone(),
                                        ).await;
                                        if success {
                                            // 成功获取歌词
                                            if config.read().await.verbose {
                                                println!("Got lyric from provider {}", provider.get_name());
                                            }
                                            // 解析歌词并且存入 lyric
                                            cache.is_lyric = true;
                                            idx = 0;
                                            timer.reset();
                                            break;
                                        } else if !try_next {
                                            // 无法获取歌词，但是不需要尝试下一个 provider
                                            // 目前此分支只会在音乐文件在 disabled_folders 中时触发
                                            break;
                                        }
                                    }
                                }
                            }
                            // println!("{:?} {:?}", lyric, cache.is_lyric);
                        }
                    }
                    Err(_) => {
                        // 播放器已经关闭。
                        if config.read().await.verbose {
                            println!("Player closed, exiting loop...");
                        }
                        tx.send(ChannelMessage::UpdateMusicInfo("".to_string(), "".to_string())).unwrap();
                        // 跳出 loop 块，继续等待下一个播放器
                        cache.player_running = false;
                        break;
                    }
                }

                // 更新播放状态、倍率和当前 mpris 进度
                let status = player.get_playback_status().unwrap_or(mpris::PlaybackStatus::Stopped);
                let playback_rate = player.get_playback_rate().unwrap_or(1.0);
                let position_micros = player.get_position().ok().map(|p| p.as_micros());

                // 上一次用于匹配歌词的时间（进度）
                let last_effective_time: u128 = timer.last_effective_time();

                // 使用计时器计算当前有效进度
                let current_time = timer.update(status, playback_rate, position_micros);

                let rate = if playback_rate <= 0.0 { 1.0 } else { playback_rate };
                let sleep_ms = ((100.0 / f64::max(rate, 1.0)) / 2.0) as u64;

                if status == mpris::PlaybackStatus::Stopped {
                    // 播放器停止：重置本地状态并清空前端显示
                    idx = 0;
                    lyric.clear();
                    timer.reset();
                    tx
                        .send(ChannelMessage::UpdateMusicInfo(
                            "".to_string(),
                            "".to_string(),
                        ))
                        .unwrap();

                    sleep(Duration::from_millis(sleep_ms)).await;
                    continue;
                }

                // 歌词是否变化？
                if cache.is_lyric {
                    // 如果进度比上一次小，重置 idx
                    if current_time < last_effective_time {
                        idx = 0;
                    }
                    if let Some(last_line) = lyric.get(idx) {
                        if current_time >= last_line.time {
                            // 歌词变化
                            while idx < lyric.len() - 1 && current_time >= lyric[idx + 1].time {
                                idx += 1;
                            }
                            let line = &lyric[idx];
                            let line_lyric = if line.tlyric.is_some() {
                                // 有翻译
                                let tlyric_clone = line.tlyric.clone().unwrap();
                                if tlyric_clone.is_empty() || tlyric_clone == line.lyric {
                                    line.lyric.clone()
                                } else {
                                    match tlyric_mode {
                                        1 => tlyric_clone,
                                        2 => format!("{} | {}", line.lyric, tlyric_clone),
                                        3 => format!("{} | {}", tlyric_clone, line.lyric),
                                        _ => line.lyric.clone(),  // 0
                                    }
                                }
                            } else {
                                // 没有翻译
                                line.lyric.clone()
                            };

                            tx.send(ChannelMessage::UpdateLyricLine(line.time, line_lyric)).unwrap();
                        }
                    }

                }

                sleep(Duration::from_millis(sleep_ms)).await;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

