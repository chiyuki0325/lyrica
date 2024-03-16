package lyric_parser

import (
	"PlasmaDesktopLyrics/types"
	"strconv"
	"strings"
)

func ParseLyrics(lyricsString string) []types.LyricLine {
	_lyrics := make([]types.LyricLine, 0)
	lines := strings.Split(lyricsString, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		lineParts := strings.Split(line, "]")
		if len(lineParts) > 1 {
			// 解析时间
			timeStr := strings.TrimPrefix(lineParts[0], "[")
			timeParts := strings.Split(timeStr, ":")
			minute, _ := strconv.Atoi(timeParts[0])
			second, _ := strconv.ParseFloat(timeParts[1], 32)
			// 解析歌词
			lyricStr := strings.TrimSpace(lineParts[1])
			_lyrics = append(_lyrics, types.LyricLine{
				Time:  int64(minute*60000000 + int(second*1000000)),
				Lyric: lyricStr,
			})
		}
	}
	return _lyrics
}
