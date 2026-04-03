use std::fmt;

#[derive(Default, Debug)]
pub(crate) struct Lyric {
    pub lines: Vec<LyricLine>,
}

#[derive(Default, Debug)]
pub struct LyricLine {
    pub time: u64, // in milliseconds
    pub text: String,
    pub alt: Option<String>, // for translation or romanization
}

impl fmt::Display for Lyric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl fmt::Display for LyricLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.time / 60000;
        let seconds = (self.time % 60000) / 1000;
        let milliseconds = self.time % 1000;
        if let Some(alt) = &self.alt {
            write!(
                f,
                "[{:02}:{:02}.{:03}] {} ({})",
                minutes, seconds, milliseconds, self.text, alt
            )
        } else {
            write!(
                f,
                "[{:02}:{:02}.{:03}] {}",
                minutes, seconds, milliseconds, self.text
            )
        }
    }
}
impl TryFrom<&str> for LyricLine {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // [01:23.45] Hello world
        // [01:23:450] Hello world (malformed but we can still parse it)

        let time_end = value.find(']').ok_or(())?;
        if !value.starts_with('[') {
            return Err(());
        }

        let time_str = &value[1..time_end];

        let time_parts = time_str.split(&[':', '.'][..]).collect::<Vec<_>>();
        if time_parts.len() < 2 {
            return Err(());
        }

        let minutes: u64 = time_parts[0].parse().map_err(|_| ())?;
        let seconds: u64 = time_parts[1].parse().map_err(|_| ())?;
        let milliseconds: u64 = if time_parts.len() > 2 {
            let ms_str = time_parts[2];
            let ms_u64 = ms_str.parse::<u64>().map_err(|_| ())?;
            if ms_str.len() == 2 {
                ms_u64 * 10
            } else {
                ms_u64
            }
        } else {
            0
        };

        let text = value[time_end + 1..].trim().to_string();

        Ok(LyricLine {
            time: minutes * 60000 + seconds * 1000 + milliseconds,
            text,
            alt: None,
        })
    }
}

impl TryFrom<String> for LyricLine {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Lyric {
    fn from_lines_iter<'a, I>(lines_iter: I) -> Result<Self, ()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut lines = Vec::<LyricLine>::new();

        for line_str in lines_iter {
            let line_trim = line_str.trim();
            if line_trim.is_empty() {
                continue;
            }

            if let Ok(lyric_line) = LyricLine::try_from(line_trim) {
                if let Some(last_line) = lines.last_mut() {
                    if last_line.time == lyric_line.time {
                        last_line.alt = Some(lyric_line.text);
                        continue;
                    }
                }
                lines.push(lyric_line);
            } else {
                if let Some(last_line) = lines.last_mut() {
                    last_line.text.push(' ');
                    last_line.text.push_str(line_trim);
                }
            }
        }
        Ok(Lyric { lines })
    }
}

impl Lyric {
    fn from_lines_iters<'a, I>(lyric_lines_iter: I, alt_lines_iter: I) -> Result<Self, ()>
    where
        I: IntoIterator<Item = &'a str>,
    {
        // parse lrc file content with translation or romanization
        let original = Self::from_lines_iter(lyric_lines_iter)?;
        let alt = Self::from_lines_iter(alt_lines_iter)?;

        let mut alt_map = std::collections::HashMap::<u64, String>::new();
        for line in alt.lines {
            alt_map.insert(line.time, line.text);
        }

        let lines = original
            .lines
            .into_iter()
            .map(|line| LyricLine {
                alt: alt_map.remove(&line.time),
                ..line
            })
            .collect();

        Ok(Lyric { lines })
    }
}

impl TryFrom<Vec<String>> for Lyric {
    type Error = ();

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Self::from_lines_iter(value.iter().map(String::as_str))
    }
}

impl TryFrom<String> for Lyric {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_lines_iter(value.lines())
    }
}

impl TryFrom<(String, String)> for Lyric {
    type Error = ();

    fn try_from(value: (String, String)) -> Result<Self, Self::Error> {
        Self::from_lines_iters(value.0.lines(), value.1.lines())
    }
}

impl TryFrom<(Vec<String>, Vec<String>)> for Lyric {
    type Error = ();

    fn try_from(value: (Vec<String>, Vec<String>)) -> Result<Self, Self::Error> {
        Self::from_lines_iters(
            value.0.iter().map(String::as_str),
            value.1.iter().map(String::as_str),
        )
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_lyric_line_parsing() {
        let line = "[01:23.45] Hello world".to_string();
        let lyric_line = LyricLine::try_from(line).unwrap();
        assert_eq!(lyric_line.time, 83_450);
        assert_eq!(lyric_line.text, "Hello world");
        assert_eq!(format!("{}", lyric_line), "[01:23.450] Hello world");
    }

    #[test]
    fn test_malformed_lyric_line_parsing() {
        let line = "[01:23:450] Hello world".to_string();
        let lyric_line = LyricLine::try_from(line).unwrap();
        assert_eq!(lyric_line.time, 83_450);
        assert_eq!(lyric_line.text, "Hello world");
        assert_eq!(format!("{}", lyric_line), "[01:23.450] Hello world");
    }
}
