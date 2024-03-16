package lyric_providers

import (
	"PlasmaDesktopLyrics/config"
	"encoding/json"
	"io"
	"net/http"
	"strings"
)

type YesPlayMusicProvider struct {
	ILyricProvider
}

func (f YesPlayMusicProvider) Initialize() {}

func (f YesPlayMusicProvider) IsMetaMode() bool {
	return false
}

func (f YesPlayMusicProvider) GetLyric(musicUrl string) (string, bool) {
	trackId := strings.TrimPrefix(musicUrl, "/trackid/")
	resp, err := http.Get("http://localhost:10754/lyric?id=" + trackId)
	if err != nil {
		return "", false
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", false
	}

	var lyricResult map[string]interface{}
	_ = json.Unmarshal(body, &lyricResult)

	_pureMusicVariant := lyricResult["pureMusic"]
	if _pureMusicVariant != nil && _pureMusicVariant.(bool) {
		return "", false
	}

	var lyricStr string
	var tLyricStr string
	if _lrcVariant := lyricResult["lrc"]; _lrcVariant != nil {
		lyricStr = _lrcVariant.(map[string]interface{})["lyric"].(string)
	}
	if _tLyricVariant := lyricResult["tlyric"]; _tLyricVariant != nil {
		tLyricStr = _tLyricVariant.(map[string]interface{})["lyric"].(string)
	}

	if config.Config.ProviderSettings.YesPlayMusic.UseTranslateLyric {
		if tLyricStr != "" {
			lyricStr = tLyricStr
		}
	}

	lyricStr = strings.Trim(lyricStr, "\n")
	if strings.HasSuffix(lyricStr, "纯音乐，请欣赏") {
		lyricStr = lyricStr[:len(lyricStr)-1]
	}

	return lyricStr, !(lyricStr == "")
}

func (f YesPlayMusicProvider) IsAvailable(musicUrl string) bool {
	if strings.HasPrefix(musicUrl, "/trackid/") {
		return true
	}
	return false
}
