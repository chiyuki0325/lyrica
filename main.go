package main

import (
	"encoding/json"
	"github.com/dhowden/tag"
	"github.com/godbus/dbus/v5"
	"github.com/gorilla/websocket"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

var (
	upgrader        = websocket.Upgrader{}
	musicUrl        string
	lyricsStr       string
	lyrics          []LyricLine
	musicInfo       MusicInfo
	isLyrics        bool
	isPlaying       bool
	currentTime     int64
	currentLyricIdx int
	ch              = make(chan []byte)
	isVerboseMode   bool
	isPlayerRunning bool
)

type LyricLine struct {
	Time  int64  `json:"time"`
	Lyric string `json:"lyric"`
}

type MusicInfo struct {
	Title  string `json:"title"`
	Artist string `json:"artist"`
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

func main() {
	// 参数检查
	if len(os.Args) > 1 {
		for _, arg := range os.Args[1:] {
			if arg == "-v" || arg == "--verbose" {
				isVerboseMode = true
			}
			if arg == "-h" || arg == "--help" {
				println("Audacious 歌词服务器")
				println("By 斬風·千雪")
				println("参数:")
				println("  -v, --verbose: 启用详细模式")
				println("  -h, --help: 显示帮助")
				println("  \n你可以访问 http://localhost:15648/test 并打开 F12 来测试。")
				return
			}
		}
	}

	upgrader.CheckOrigin = func(r *http.Request) bool { return true }
	http.HandleFunc("/ws", handleUpgradeWebSocket)
	http.HandleFunc("/test", testHtml)
	go update()
	println("在 http://localhost:15648 启动了 audacious-lyricsd 服务器")
	_ = http.ListenAndServe(":15648", nil)
}

func handleUpgradeWebSocket(w http.ResponseWriter, r *http.Request) {
	// 升级到 websocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		http.Error(w, "升级到 websocket 失败", http.StatusInternalServerError)
		return
	}
	println("有新的连接")
	// 清空缓存
	for len(ch) > 0 {
		<-ch
	}
	defer conn.Close()
	for {
		msg := <-ch
		_ = conn.SetWriteDeadline(time.Now().Add(time.Second))
		err = conn.WriteMessage(websocket.TextMessage, msg)
		if isVerboseMode {
			println(string(msg))
		}
		if err != nil {
			if isVerboseMode {
				println("连接断开")
			}
			conn.Close()
			break
		}
	}
}

func parseLyrics() {
	lyrics = make([]LyricLine, 0)
	lines := strings.Split(lyricsStr, "\n")
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
			lyrics = append(lyrics, LyricLine{
				Time:  int64(minute*60000000 + int(second*1000000)),
				Lyric: lyricStr,
			})
		}
	}
}

func update() {
	d, _ := dbus.ConnectSessionBus()
	defer d.Close()
	for {
		// 处理更新
		// 1: 每一秒更新，获取当前歌曲
		dbusObj := d.Object("org.mpris.MediaPlayer2.audacious", "/org/mpris/MediaPlayer2")
		dbusPropObj, err := dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.Metadata")
		if err != nil {
			isPlayerRunning = false
		} else {
			isPlayerRunning = true
		}

		newMusicUrl := ""
		if isPlayerRunning {
			_newMusicUrlVariant := dbusPropObj.Value()
			if _newMusicUrlVariant == nil {
				newMusicUrl = ""
				isPlayerRunning = false
			} else {
				newMusicUrl = _newMusicUrlVariant.(map[string]dbus.Variant)["xesam:url"].Value().(string)
			}
		}

		if newMusicUrl != musicUrl {
			musicUrl = newMusicUrl
			if !strings.HasPrefix(newMusicUrl, "file://") {
				// 不是本地文件 或者 播放器未运行 或者 没有歌曲
				if isVerboseMode {
					println("状态更新，但不是本地文件，或者播放器未运行，或者没有歌曲")
				}
				musicInfo = MusicInfo{Title: "", Artist: ""}
				isLyrics = false
			} else {
				// 歌曲更换
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
				// 此时如果有歌词则 lyrics 变量不为空

				// 获取歌曲信息
				musicInfo = MusicInfo{Title: metadata.Title(), Artist: metadata.Artist()}
			}
			// 此时 musicInfo 和 isLyrics 都应该已经更新
			b, _ := json.Marshal(musicInfo)
			ch <- b
			// 解析歌词
			if isLyrics {
				parseLyrics()
			} else {
				// 清空歌词
				b, _ := json.Marshal(LyricLine{Time: -1, Lyric: ""})
				ch <- b
			}
		}

		i := 0
		for i = 0; i < 40; i++ {
			if isPlayerRunning && isLyrics {
				// 每 25ms 更新歌词
				var playbackStatus string
				_dbusResult, err := dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.PlaybackStatus")
				if err != nil {
					// 播放器突然关闭了
					if isVerboseMode {
						println("播放器关闭了，歌词更新终止。")
					}
					isPlayerRunning = false
					break
				}

				playbackStatus = _dbusResult.Value().(string)
				if playbackStatus == "Playing" {
					isPlaying = true
				} else {
					isPlaying = false
				}
				_dbusResult, _ = dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.Position")
				_currentTime := _dbusResult.Value().(int64)
				if _currentTime < currentTime {
					// 换歌了，退出循环
					currentTime = _currentTime
					break
				}
				currentTime = _currentTime
				if isPlaying && isLyrics {
					// 有歌词
					idx := 0
					for _, lyric := range lyrics {
						if lyric.Time >= currentTime {
							break
						}
						idx++
					}
					idx -= 1
					if currentLyricIdx != idx {
						if idx < 0 {
							idx = 0
						}
						currentLyricIdx = idx
						// 发送歌词
						b, _ := json.Marshal(lyrics[idx])
						ch <- b
					}
				}
			}
			time.Sleep(time.Millisecond * 25)
		}
	}
}

func testHtml(w http.ResponseWriter, r *http.Request) {
	w.Write([]byte(`
<body>
<script>
    ws = new WebSocket("ws://localhost:15648/ws")
    ws.onmessage = function(event) {
        console.log(event.data)
    }
	ws.onopen = function(event) {
		console.log("连接成功")
		ws.send("hello")
	}
</script>
</body>
`))
}
