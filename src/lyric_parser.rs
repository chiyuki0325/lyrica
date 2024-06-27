pub struct LyricLine {
    pub time: u128,
    pub lyric: String,
    pub tlyric: Option<String>,
}

pub(crate) fn parse_lyrics(lyric_string: String) -> Vec<LyricLine> {
    let mut lyrics: Vec<LyricLine> = Vec::new();
    let lines = lyric_string.lines();

    let mut last_time = 0;

    for line in lines {
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
