package lyric_providers

import (
	"PlasmaDesktopLyrics/config"
	"github.com/dhowden/tag"
	"net/url"
	"os"
	"path/filepath"
	"strings"
)

type FileLyricProvider struct {
	ILyricProvider
}

func parseFileUrl(urlStr string) (filePath string, err error) {
	// 解析URL
	u, err := url.Parse(urlStr)
	if err != nil {
		return "", err
	}

	// 解码路径
	filePath, err = url.PathUnescape(u.Path)
	if err != nil {
		return "", err
	}

	return filePath, nil
}

func (f FileLyricProvider) GetLyric(musicUrl string) (string, bool) {
	isLyrics := false
	lyricsStr := ""
	// (1) 解析歌曲文件头是否有歌词
	musicFilePath, _ := parseFileUrl(musicUrl)
	file, _ := os.Open(musicFilePath)
	metadata, _ := tag.ReadFrom(file)
	_lyrics := metadata.Lyrics()
	if !(_lyrics == "") {
		isLyrics = true
		lyricsStr = _lyrics
	} else {
		// (2) 查找是否有 lrc 文件
		lyricsFilePath := strings.TrimSuffix(musicFilePath, filepath.Ext(musicFilePath)) + ".lrc"
		_, err := os.Stat(lyricsFilePath)
		if err == nil {
			// 有 lrc 文件
			_lyrics, _ := os.ReadFile(lyricsFilePath)
			lyricsStr = string(_lyrics)
			isLyrics = true
		} else {
			// 无 lrc 文件
			lyricsStr = ""
			isLyrics = false
		}
	}
	if config.Config.Verbose {
		if isLyrics {
			println("使用 file 成功获取到歌词")
		} else {
			println("使用 file 未能获取到歌词")
		}
	}
	return lyricsStr, isLyrics
}

func (f FileLyricProvider) IsAvailable(musicUrl string) bool {
	if strings.HasPrefix(musicUrl, "file://") {
		if config.Config.Verbose {
			println("file 可以处理此 URL")
		}
		return true
	}
	if config.Config.Verbose {
		println("file 不能处理此 URL")
	}
	return false
}
