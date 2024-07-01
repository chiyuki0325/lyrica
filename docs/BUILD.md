# Build Instructions

### KDE Plasma

Lyrica only works with Plasma version 6.0 or later.

#### Install dependencies

Debian/Ubuntu:

```bash
sudo apt install rustup jq qt6-declarative-dev qt6-websockets-dev qml6-module-qtwebsockets
rustup toolchain install stable
``` 

Arch Linux:

```bash
sudo pacman -S --needed rustup qt6-declarative qt6-websockets jq
rustup toolchain install stable
````

Fedora:

```bash
sudo dnf install rustup jq qt6-qtdeclarative qt6-qtdeclarative-devel qt6-qtwebsockets qt6-qtwebsockets-devel
rustup toolchain install stable
```

openSUSE:

```bash
sudo zypper install rustup jq qt6-declarative qt6-websockets qt6-websockets-imports 
rustup toolchain install stable
````

#### Build

```bash
bash build_plasmoid.sh [architecture]
```

#### Install
```bash
kpackagetool6 -i lyrica-plasmoid-<version>-<arch>.plasmoid -t Plasma/Applet
```
