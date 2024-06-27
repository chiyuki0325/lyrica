# KDE Plasma Desktop Lyrics Widget

# The widget is being rewritten in Rust and the current version will be soon deprecated.

# Check [next branch](https://github.com/chiyuki0325/PlasmaDesktopLyrics/tree/next) for the new version.

[简体中文](./README_CN.md)

## Daemon

### Build & Install

```bash
go build -o ~/.local/bin/plasma-desktop-lyrics-daemon
cat misc/plasma-desktop-lyrics.service | sed "s#USERNAME#$USER#" > ~/.config/systemd/user/plasma-desktop-lyrics.service
systemctl enable --now --user plasma-desktop-lyrics.service
```

### Uninstall

```bash
killall plasma-desktop-lyrics-daemon
rm ~/.local/bin/plasma-desktop-lyrics-daemon
systemctl disable --now --user plasma-desktop-lyrics
rm ~/.config/systemd/user/plasma-desktop-lyrics.service
```

## Plasma Widget

### Install

```bash
cp -r plasmoid ~/.local/share/plasma/plasmoids/ink.chyk.plasmaDesktopLyrics
systemctl restart --user plasma-plasmashell.service
```

### Uninstall

```bash
rm -rf ~/.local/share/plasma/plasmoids/ink.chyk.audaciousLyrics
systemctl restart --user plasma-plasmashell.service
```

