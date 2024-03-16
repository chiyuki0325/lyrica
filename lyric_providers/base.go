package lyric_providers

type ILyricProvider interface {
	GetLyric(musicUrl string) (string, bool)
	IsAvailable(musicUrl string) bool
}

var LyricProviders = map[string]ILyricProvider{
	"file": FileLyricProvider{},
}
