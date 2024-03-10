# KDE Audacious 歌词显示组件

## 构建并安装守护进程

```bash
go build -o ~/.local/bin/audacious-lyricsd
cat audacious-lyricsd.service | sed "s#USERNAME#$USER#" > ~/.config/systemd/user/audacious-lyricsd.service
systemctl enable --now --user audacious-lyricsd
```

## 卸载守护进程

```bash
killall audacious-lyricsd
rm ~/.local/bin/audacious-lyricsd
systemctl disable --now --user audacious-lyricsd
rm ~/.config/systemd/user/audacious-lyricsd.service
```

## 安装顶栏控件

```bash
cp -r plasmoid ~/.local/share/plasma/plasmoids/ink.chyk.audaciousLyrics
systemctl restart --user plasma-plasmashell
```

## 卸载顶栏控件

```bash
rm -rf ~/.local/share/plasma/plasmoids/ink.chyk.audaciousLyrics
systemctl restart --user plasma-plasmashell
```

## 构建桌面歌词程序

```bash
cd AudaciousLyricsQML
mkdir build; cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_QML=ON
make
sudo install -Dm755 AudaciousLyricsQML /usr/local/bin/AudaciousLyricsQML
sudo install -Dm644 ../../audacious-desktop-lyrics.desktop /usr/share/applications/audacious-desktop-lyrics.desktop
```

### 卸载桌面歌词程序

```bash
sudo rm /usr/share/applications/audacious-desktop-lyrics.desktop
sudo rm /usr/local/bin/AudaciousLyricsQML
```

