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
        let time_parts: Vec<&str> = time_str.split(':').collect();
        let minute: i64 = time_parts[0].parse().unwrap_or(0);
        let second: f64 = time_parts[1].parse().unwrap_or(0.0);

        // 解析歌词
        let lyric_str = line_parts[1].trim().to_string();
        let time = ((minute * 60000000) as u128 + (second * 1000000.0) as u128);
        return Ok((time, lyric_str));
    }
    Err(())
}

pub(crate) fn parse_lyrics(lyric_string: String) -> Vec<LyricLine> {
    let mut lyrics: Vec<LyricLine> = Vec::new();
    let lines = lyric_string.lines();

    let mut last_time = 0;

    for line in lines {
        if let Ok((time, lyric_str)) = parse_single_line(String::from(line)) {
            if last_time == time {
                if let Some(last_lyric) = lyrics.last_mut() {
                    last_lyric.tlyric = Some(lyric_str);
                }
            } else {
                lyrics.push(LyricLine {
                    time,
                    lyric: lyric_str,
                    tlyric: None,
                });
            }
            last_time = time;
        }
    }

    lyrics
}


pub(crate) fn parse_netease_lyrics(
    lyric_lines: Vec<String>,
    tlyric_lines: Vec<String>,
) -> Vec<LyricLine> {
    // for netease provider
    let mut lyrics: Vec<LyricLine> = Vec::new();

    for line in lyric_lines {
        if let Ok((time, lyric_str)) = parse_single_line(line) {
            lyrics.push(LyricLine {
                time,
                lyric: lyric_str,
                tlyric: None,
            });
        }
    }

    let mut last_idx = 0;

    for line in tlyric_lines {
        if let Ok((time, lyric_str)) = parse_single_line(line) {
            let mut idx = last_idx;
            while idx < lyrics.len() && lyrics[idx].time < time {
                idx += 1;
            }
            // 此时应该等于
            if let Some(lyric_line) = lyrics.get_mut(idx) {
                if lyric_line.time == time {
                    lyric_line.tlyric = Some(lyric_str);
                    last_idx = idx;
                }
            }
        }
    }

    return lyrics;
}
