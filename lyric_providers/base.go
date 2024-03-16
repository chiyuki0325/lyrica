package lyric_providers

import "github.com/godbus/dbus/v5"

type ILyricProvider interface {
	Initialize()
	GetLyric(musicUrl string) (string, bool)
	GetLyricByMeta(musicMeta map[string]dbus.Variant) (string, bool)
	IsAvailable(musicUrl string) bool
	IsMetaMode() bool
}

var LyricProviders = map[string]ILyricProvider{
	"file":         FileLyricProvider{},
	"yesplaymusic": YesPlayMusicProvider{},
	"netease":      NetEaseLyricProvider{},
}
