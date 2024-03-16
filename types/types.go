package types

type LyricLine struct {
	Time  int64  `json:"time"`
	Lyric string `json:"lyric"`
}

type MusicInfo struct {
	Title  string `json:"title"`
	Artist string `json:"artist"`
}
