package main

import (
	"PlasmaDesktopLyrics/config"
	"PlasmaDesktopLyrics/lyric_parser"
	"PlasmaDesktopLyrics/lyric_providers"
	"PlasmaDesktopLyrics/mpris"
	"PlasmaDesktopLyrics/types"
	"encoding/json"
	"github.com/godbus/dbus/v5"
	"github.com/gorilla/websocket"
	"net/http"
	"os"
	"slices"
	"time"
)

var (
	upgrader          = websocket.Upgrader{}
	musicUrl          string
	lyricsStr         string
	lyrics            []types.LyricLine
	player            string
	isLyrics          bool
	isPlaying         bool
	currentTime       int64
	currentLyricIdx   int
	ch                = make(chan []byte)
	isPlayerRunning   bool
	isPlayerPlaying   bool
	isClientReceiving bool = false
)

func main() {
	// 加载配置文件
	config.LoadConfig()

	// 参数检查
	if len(os.Args) > 1 {
		for _, arg := range os.Args[1:] {
			if arg == "-v" || arg == "--verbose" {
				config.Config.Verbose = true
			}
			if arg == "-h" || arg == "--help" {
				println("KDE Plasma 桌面歌词服务器")
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
	go updateMeta()
	go updateLyrics()
	println("在 http://localhost:15648 启动了桌面歌词服务器")
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
	isClientReceiving = true
	defer conn.Close()
	for {
		msg := <-ch
		_ = conn.SetWriteDeadline(time.Now().Add(time.Second))
		err = conn.WriteMessage(websocket.TextMessage, msg)
		if config.Config.Verbose {
			println(string(msg))
		}
		if err != nil {
			if config.Config.Verbose {
				println("连接断开")
			}
			conn.Close()
			break
		}
	}
}

func updateMeta() {
	d, _ := dbus.ConnectSessionBus()
	defer d.Close()
	for {
		if !isClientReceiving {
			// 客户端未在运行，等待，防止频道中积压过多歌词
			time.Sleep(time.Second)
			continue
		}

		_isPlayerRunning := true

		// 获取播放器
		players := mpris.ListPlayers(d)

		if len(players) == 0 {
			// 没有播放器
			_isPlayerRunning = false
		}

		var musicMeta map[string]dbus.Variant

		if _isPlayerRunning {
			player = players[0]
			var ret bool
			musicMeta, ret = mpris.GetMetadata(d, player)

			if !ret {
				_isPlayerRunning = false
			}
		}

		if _isPlayerRunning != isPlayerRunning {
			isPlayerRunning = _isPlayerRunning
			if !_isPlayerRunning {
				// 播放器关闭了，流程终止
				b, _ := json.Marshal(types.MusicInfo{Title: "", Artist: ""})
				ch <- b
				b, _ = json.Marshal(types.LyricLine{Time: -1, Lyric: ""})
				ch <- b
				isPlayerRunning = false
				continue
			}
		}

		if isPlayerRunning {
			var newMusicUrl string
			if _newmusicurlVariant := musicMeta["xesam:url"].Value(); _newmusicurlVariant != nil {
				newMusicUrl = _newmusicurlVariant.(string)
			} else if _newmusicurlVariant = musicMeta["mpris:artUrl"].Value(); _newmusicurlVariant != nil {
				newMusicUrl = _newmusicurlVariant.(string)
			} else {
				// 无法获取到歌曲 URL
				newMusicUrl = ""
			}

			if newMusicUrl != musicUrl {
				if newMusicUrl == "" {
					musicUrl = ""
					b, _ := json.Marshal(types.MusicInfo{Title: "", Artist: ""})
					ch <- b
					b, _ = json.Marshal(types.LyricLine{Time: -1, Lyric: ""})
					ch <- b
				} else {
					// 歌曲更换
					musicUrl = newMusicUrl
					if config.Config.Verbose {
						println("歌曲更换: " + musicUrl)
					}

					// 更新信息
					_t := musicMeta["xesam:title"].Value()
					_a := musicMeta["xesam:artist"].Value()
					if _t == nil || _a == nil {
						// 播放的不是音乐，或者播放的音乐没有数据标签
						isPlayerPlaying = false
						b, _ := json.Marshal(types.MusicInfo{Title: "", Artist: ""})
						ch <- b
						b, _ = json.Marshal(types.LyricLine{Time: -1, Lyric: ""})
						ch <- b
						continue
					}

					b, _ := json.Marshal(types.MusicInfo{
						Title:  _t.(string),
						Artist: _a.([]string)[0],
					})
					ch <- b
					isPlayerPlaying = true

					isLyrics = false
					for key, provider := range lyric_providers.LyricProviders {
						if slices.Contains(config.Config.EnabledLyricProviders, key) {
							if provider.IsAvailable(newMusicUrl) {
								if provider.IsMetaMode() {
									if config.Config.Verbose {
										println("尝试使用 " + key + " 和元数据获取歌词")
									}
									lyricsStr, isLyrics = provider.GetLyricByMeta(musicMeta)
								} else {
									if config.Config.Verbose {
										println("尝试使用 " + key + " 获取歌词")
									}
									lyricsStr, isLyrics = provider.GetLyric(newMusicUrl)
								}
								if isLyrics {
									if config.Config.Verbose {
										println("获取歌词成功")
									}
									break
								}
							} else {
								if config.Config.Verbose {
									println(key + " 不能获取此歌曲的歌词")
								}
							}
						}
					}
				}

				if isLyrics {
					lyrics = lyric_parser.ParseLyrics(lyricsStr)
				} else {
					b, _ := json.Marshal(types.LyricLine{Time: -1, Lyric: ""})
					ch <- b
				}
			}
		}
		time.Sleep(time.Millisecond * 500)
	}
}

func updateLyrics() {
	d, _ := dbus.ConnectSessionBus()
	defer d.Close()
	for {
		i := 0
		for i = 0; i < 40; i++ {
			if isPlayerRunning && isPlayerPlaying && isLyrics {
				// 每 25ms 更新歌词
				playbackStatus, err := mpris.GetPlaybackStatus(d, player)
				if err != nil || playbackStatus == "" {
					// 播放器突然关闭了，或是启动了新的播放器但是未开始播放
					if config.Config.Verbose {
						println("播放器关闭了")
					}
					isPlayerPlaying = false
					b, _ := json.Marshal(types.LyricLine{Time: -1, Lyric: ""})
					ch <- b
					b, _ = json.Marshal(types.MusicInfo{Title: "", Artist: ""})
					ch <- b
					break
				}

				if playbackStatus == "Playing" {
					isPlaying = true
				} else {
					isPlaying = false
				}

				_currentTime := mpris.GetPosition(d, player)
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
