# 歌词源

Lyrica 使用以下歌词源来获取播放器的歌词。

可以在 Lyrica 小部件的设置中禁用不需要的歌词源。

## `mpris2_text`

如果你的播放器支持在 mpris2 元数据的 asText 中提供带时间轴的歌词，那么这个歌词源将被使用。

## `file`

这个歌词源从音乐文件的标签或 LRC 文件中读取歌词，当播放本地音乐时使用。

支持的文件格式：mp3、flac、lrc。

## `yesplaymusic`

YesPlayMusic 是网易云音乐的第三方播放器。当使用此播放器时，Lyrica 可以从 YesPlayMusic 提供的 API 获取歌词。

## `netease_gtk4`

NetEase Cloud Music GTK4 是另一个流行的网易云音乐第三方播放器。当使用 NetEase Cloud Music GTK4 时，可以使用此歌词源。

## `feeluown_netease`

FeelUOwn 是一个用 Python 编写的本地 / 在线音乐播放器。当使用 FeelUOwn 的网易源听歌时，可以使用此歌词源。

## `netease`

`netease` 歌词源会在网易云音乐中查找对应的歌曲并获取歌词。当其他歌词源不可用时，可以作为备用选项使用。
