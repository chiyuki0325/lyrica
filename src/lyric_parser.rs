#[derive(Debug)]
pub struct LyricLine {
    pub time: u128,
    pub lyric: String,
    pub tlyric: Option<String>,
}


fn parse_single_line(line: String) -> Result<(u128, String), ()> {
    // 解析一行歌词
    let line = line.trim();
    let line_parts: Vec<&str> = line.split(']').collect();

    if line_parts.len() > 1 {
        // 解析时间
        let time_str = line_parts[0].trim_start_matches('[');
        let (minute, second) = match time_str.split(':').collect::<Vec<_>>().as_slice() {
            [min_str, sec_str] => {
                let minute: i64 = min_str.parse().unwrap_or(0);
                let second: f64 = sec_str.replace(",", ".").parse().unwrap_or(0.0);
                (minute, second)
            }
            [min_str, sec_str, subsec_str] => {
                let minute: i64 = min_str.parse().unwrap_or(0);
                let sec: f64 = sec_str.parse().unwrap_or(0.0);
                let subsec: f64 = subsec_str.parse().unwrap_or(0.0);
                let scale = 10f64.powi(subsec_str.len() as i32);
                let second: f64 = sec + subsec / scale;
                (minute, second)
            }
            _ => return Err(()),
        };

        // 解析歌词
        let lyric_str = line_parts[1].trim().to_string();
        let time = (minute * 60000000) as u128 + (second * 1000000.0) as u128;
        return Ok((time, lyric_str));
    }
    Err(())
}

pub(crate) fn parse_lyrics(lyric_string: String) -> Vec<LyricLine> {
    let lyric_lines = lyric_string.lines();
    let mut lyrics: Vec<LyricLine> = Vec::new();

    for line in lyric_lines {
        if let Ok((time, lyric_str)) = parse_single_line(String::from(line)) {

            // println!("line {}: {}", time, lyric_str);
            let mut idx = 0;
            loop {
                if let Some(lyric_line) = lyrics.get(idx) {
                    if lyric_line.time == time {
                        // 这句歌词是该歌词的翻译
                        // println!("TLYRIC {}: {}", time, lyric_str);
                        lyrics[idx].tlyric = Some(lyric_str);
                        break;
                    } else if time < lyric_line.time {
                        // 是新的一句歌词
                        // println!("APPEND {}: {}", time, lyric_str);
                        lyrics.push(LyricLine {
                            time,
                            lyric: lyric_str,
                            tlyric: None,
                        });
                        break;
                    } else {
                        // println!("SKIPPING {}: {}", lyric_line.time, lyric_line.lyric);
                        idx += 1;
                    }
                } else {
                    // println!("APPEND {}: {}", time, lyric_str);
                    lyrics.push(LyricLine {
                        time,
                        lyric: lyric_str.clone(),
                        tlyric: None,
                    });
                    break;
                }
            }
        }
    }

    lyrics
}

pub fn merge_lyrics(mut lyrics: Vec<LyricLine>, tlyrics: Vec<LyricLine>) -> Vec<LyricLine> {
    let mut j = 0;

    for l in lyrics.iter_mut() {
        while j < tlyrics.len() && tlyrics[j].time < l.time {
            j += 1;
        }
        if j < tlyrics.len() && tlyrics[j].time == l.time {
            l.tlyric = Some(tlyrics[j].lyric.clone());
        }
    }

    lyrics
}

pub(crate) fn parse_netease_lyrics(
    lyric_lines: Vec<String>,
    tlyric_lines: Vec<String>,
) -> Vec<LyricLine> {
    if !tlyric_lines.is_empty() {
        let lyrics = parse_lyrics(lyric_lines.join("\n"));
        let tlyrics = parse_lyrics(tlyric_lines.join("\n"));
        merge_lyrics(lyrics, tlyrics)
    } else {
        parse_lyrics(lyric_lines.join("\n"))
    }
}
