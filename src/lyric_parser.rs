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
    let lyric_lines = lyric_string.lines();
    let mut lyrics: Vec<LyricLine> = Vec::new();

    for line in lyric_lines {
        if let Ok((time, lyric_str)) = parse_single_line(String::from(line)) {

            let mut idx = 0;
            loop {
                if let Some(lyric_line) = lyrics.get(idx) {
                    if lyric_line.time == time {
                        // 这句歌词是该歌词的翻译
                        lyrics[idx].tlyric = Some(lyric_str);
                        break;
                    } else if time < lyric_line.time {
                        // 是新的一句歌词
                        lyrics.push(LyricLine {
                            time,
                            lyric: lyric_str,
                            tlyric: None,
                        });
                        break;
                    } else {
                        idx += 1;
                    }
                } else {
                    lyrics.push(LyricLine {
                        time,
                        lyric: lyric_str.clone(),
                        tlyric: None,
                    });
                }
            }
        }
    }

    lyrics
}


pub(crate) fn parse_netease_lyrics(
    lyric_lines: Vec<String>,
    tlyric_lines: Vec<String>,
) -> Vec<LyricLine> {
    parse_lyrics(
        lyric_lines.join("\n") + "\n" + &tlyric_lines.join("\n")
    )
}
