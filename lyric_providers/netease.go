package lyric_providers

import (
	"PlasmaDesktopLyrics/config"
	"encoding/json"
	"github.com/go-musicfox/netease-music/service"
	"github.com/go-musicfox/netease-music/util"
	"github.com/godbus/dbus/v5"
	"github.com/telanflow/cookiejar"
	"strconv"
	"strings"
)

type NetEaseLyricProvider struct{}

var cookieJar = cookiejar.Jar{}

func (n NetEaseLyricProvider) Initialize() {
	// 初始化 cookieJa
	util.SetGlobalCookieJar(&cookieJar)
}
func (n NetEaseLyricProvider) IsMetaMode() bool {
	return true
}

func (n NetEaseLyricProvider) IsAvailable(musicUrl string) bool {
	return true
}
func (n NetEaseLyricProvider) GetLyric(musicUrl string) (string, bool) {
	return "", false
}

func (n NetEaseLyricProvider) GetLyricByMeta(musicMeta map[string]dbus.Variant) (string, bool) {
	songName := musicMeta["xesam:title"].Value().(string)
	songArtist := musicMeta["xesam:artist"].Value().([]string)[0]

	searchApi := service.SearchService{
		S:     songName + " " + songArtist,
		Limit: "5",
	}
	_, body := searchApi.Search()
	var searchResult map[string]interface{}
	_ = json.Unmarshal(body, &searchResult)

	song := searchResult["result"].(map[string]interface{})["songs"].([]interface{})[0].(map[string]interface{})

	if strings.ToLower(song["name"].(string)) != strings.ToLower(musicMeta["xesam:title"].Value().(string)) {
		return "", false
	}

	musicLength := musicMeta["mpris:length"].Value().(int64) / 1000000
	searchedMusicLength := int64(song["dt"].(float64))
	// 如果相差大于 6 秒，则认为不是同一首歌
	if musicLength < searchedMusicLength-6 || musicLength > searchedMusicLength+6 {
		return "", false
	}

	songId := strconv.FormatFloat(song["id"].(float64), 'f', -1, 64)

	if config.Config.Verbose {
		println("网易云音乐歌曲 ID: " + songId)
	}

	lyricApi := service.LyricService{
		ID: songId,
	}

	_, lyricBody := lyricApi.Lyric()

	var lyricResult map[string]interface{}
	_ = json.Unmarshal(lyricBody, &lyricResult)

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

	if config.Config.ProviderSettings.NetEase.UseTranslateLyric {
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
