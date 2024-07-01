# Build Instructions

### KDE Plasma

Lyrica only works with Plasma version 6.0 or later.

#### Fetch source code

```bash
git clone https://github.com/chiyuki0325/lyrica
cd lyrica
git checkout v0.3
```

#### Install dependencies

Debian/Ubuntu:

```bash
sudo apt install rustup jq qt6-declarative-dev qt6-websockets-dev qml6-module-qtwebsockets libdbus-1-dev
rustup toolchain install stable
``` 

Arch Linux:

```bash
sudo pacman -S --needed rustup qt6-declarative qt6-websockets jq
rustup toolchain install stable
````

Fedora:

```bash
sudo dnf install rustup jq qt6-qtdeclarative qt6-qtdeclarative-devel qt6-qtwebsockets qt6-qtwebsockets-devel dbus-devel
rustup toolchain install stable
```

openSUSE:

```bash
sudo zypper install rustup jq qt6-declarative qt6-websockets qt6-websockets-imports dbus-1-devel
rustup toolchain install stable
````

#### Build

```bash
bash build_plasmoid.sh
```

#### Install
```bash
kpackagetool6 -i lyrica-plasmoid-<version>-<arch>.plasmoid -t Plasma/Applet
```

#### Upgrade
```bash
kpackagetool6 -u lyrica-plasmoid-<version>-<arch>.plasmoid -t Plasma/Applet
```
