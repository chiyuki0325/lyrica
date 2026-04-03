# 构建指南

### KDE Plasma

Lyrica 仅支持 Plasma 6.0 或更高版本。

由于本项目使用相对路径运行后端，因此不支持通过 Flatpak 或 Snap 安装的桌面环境，也不支持如 NixOS 等使用非标准家目录的发行版。

#### 获取源代码

```bash
git clone https://github.com/chiyuki0325/lyrica --branch=v1
cd lyrica
git checkout $(git tag --list "v*" | tail -1)
```

#### 安装依赖

**Debian/Ubuntu:**

```bash
sudo apt install rustup jq qt6-declarative-dev qt6-websockets-dev qml6-module-qtwebsockets libdbus-1-dev
rustup toolchain install stable
```

**Arch Linux:**

```bash
sudo pacman -S --needed rustup qt6-declarative qt6-websockets jq
rustup toolchain install stable
```

**Fedora:**

```bash
sudo dnf install rustup jq qt6-qtdeclarative qt6-qtdeclarative-devel qt6-qtwebsockets qt6-qtwebsockets-devel dbus-devel
rustup toolchain install stable
```

**openSUSE:**

```bash
sudo zypper install rustup jq qt6-declarative qt6-websockets qt6-websockets-imports dbus-1-devel
rustup toolchain install stable
```

#### 构建

```bash
bash frontend/build_plasmoid.sh
```

#### 安装

```bash
kpackagetool6 -i lyrica-plasmoid-<版本>-<发行版>-<架构>.plasmoid -t Plasma/Applet
```

#### 升级

```bash
kpackagetool6 -u lyrica-plasmoid-<版本>-<发行版>-<架构>.plasmoid -t Plasma/Applet
```