<div align="center">
  <img src="assets/lyrica.png" width="128px">
  <br>  
    <h2>Lyrica</h2>
</div>

[简体中文](/README.zh.md)

Lyrica is a simple software to show desktop lyrics on Linux focused on simplicity and integration.  
Unlike similar projects like [Waylyrics](https://github.com/waylyrics/waylyrics), it integrates with desktop environments and shows lyrics on the desktop's panels.  
Currently the project is in development, but the current version is already usable.  

The project's name comes from the Touhou Project character Lyrica Prismriver.

### Project status

- [x] LRC parser
- [x] Real-time lyrics
- [x] File lyric provider
- [x] NetEase Cloud Music lyric provider
  - [ ] Different search patterns
  - [ ] Cache online lyrics
- [ ] Player API lyric provider
  - [ ] Mpris2 asText
  - [ ] YesPlayMusic
  - [ ] MusicFox, etc.
- [x] KDE Plasma Plasmoid frontend
- [ ] ~~GNOME shell extension frontend~~
  There are also similar projects like desktop-lyrics and osdlyrics, so GNOME shell extension is not a priority.
- [ ] User documents


### Usage

#### KDE Plasma

Build:
```bash
bash build_plasmoid.sh [architecture]
```
Or download the latest release from the [release page](https://github.com/chiyuki0325/lyrica/releases).

Install:
```bash
kpackagetool6 -i lyrica-plasmoid-<arch>.plasmoid -t Plasma/Applet
```

The logo of this project comes from the "東方 Project リバイバルちほー" of the rhythm game "maimai でらっくす".
Copyright belongs to the original author.
