# Lyric Providers

[简体中文](LYRIC_PROVIDERS.zh.md)

The lyric providers below are used to fetch lyrics from players.

You can disable unnecessary lyric providers in the settings of the Lyrica widget.

## `Mpris2Text`

If your player supports providing lyrics with timeline in asText of mpris2 metadata, then this lyric provider will be used.

## `File`

This lyrics provider reads lyrics from the music file's tags or LRC files when playing local music.

Supports the following file formats: mp3, flac, lrc.

## `YesPlayMusic`

YesPlayMusic is a third-party player for NetEase Cloud Music. Lyrica can get lyrics from the API provided by YesPlayMusic when using this player.

## `NeteaseTrackID`

This lyric provider is available to third-party NetEase Cloud Music players that expose music track IDs like ElectronNCM and NetEase Cloud Music GTK4.

## `FeelUOwnNetease`

FeelUOwn is a local & online music player written in Python. This lyrics provider is available when using FeelUOwn to listen to songs on NetEase Cloud Music.

## `Netease`

The `Netease` lyrics provider finds the corresponding song and obtains the lyrics from NetEase Cloud Music. It can be used as a fallback option when other lyrics providers are unavailable.
