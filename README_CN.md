# KDE Plasma 桌面歌词组件

## 构建并安装守护进程

```bash
go build -o ~/.local/bin/plasma-desktop-lyrics-daemon
cat misc/plasma-desktop-lyrics.service | sed "s#USERNAME#$USER#" > ~/.config/systemd/user/plasma-desktop-lyrics.service
systemctl enable --now --user plasma-desktop-lyrics.service
```

## 卸载守护进程

```bash
killall plasma-desktop-lyrics-daemon
rm ~/.local/bin/plasma-desktop-lyrics-daemon
systemctl disable --now --user plasma-desktop-lyrics
rm ~/.config/systemd/user/plasma-desktop-lyrics.service
```

## 安装控件

```bash
cp -r plasmoid ~/.local/share/plasma/plasmoids/ink.chyk.plasmaDesktopLyrics
systemctl restart --user plasma-plasmashell.service
```

## 卸载控件

```bash
rm -rf ~/.local/share/plasma/plasmoids/ink.chyk.audaciousLyrics
systemctl restart --user plasma-plasmashell.service
```

