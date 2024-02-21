# KDE Audacious 歌词显示组件

## 构建并安装

```bash
go build -o ~/.local/bin/audacious-lyricsd
cat audacious-lyricsd.service | sed "s#USERNAME#$USER#" > ~/.config/systemd/user/audacious-lyricsd.service
kpackagetool5 -i plasmoid
systemctl enable --now --user audacious-lyricsd
systemctl restart --user plasma-plasmashell
```

## 卸载

```bash
killall audacious-lyricsd
rm ~/.local/bin/audacious-lyricsd
systemctl disable --now --user audacious-lyricsd
rm ~/.config/systemd/user/audacious-lyricsd.service
kpackagetool5 -u plasmoid
systemctl restart --user plasma-plasmashell
```

