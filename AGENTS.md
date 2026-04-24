# AGENTS.md

Guidelines for AI agents working in this repository.

**Maintenance rule:** Any time you add or modify a feature — new lyric provider, new config field, new WebSocket message type, changed API endpoint, changed behavior — you must update the relevant sections of this file in the same change. Do not defer documentation to a follow-up commit.

## Project Overview

**Lyrica** is a lightweight Linux desktop lyrics display application. It consists of:

- A **Rust backend** daemon that discovers MPRIS2 music players via D-Bus, fetches lyrics from multiple sources, synchronizes lyric lines in real time, and exposes results over a WebSocket/HTTP server on `127.0.0.1:15650`.
- A **KDE Plasma 6 Plasmoid** (QML) that connects to the backend and renders the current lyric line on the panel.
- An **OBS Studio plugin** (Python) that similarly consumes the WebSocket feed.

## Repository Layout

```
src/                      # Rust backend
  main.rs                 # Entry point: HTTP server, MPRIS loop spawn
  config.rs               # Config struct and persistence
  state.rs                # Shared app state + broadcast channel
  messages.rs             # ChannelMessage enum (WebSocket payloads)
  lyric_parser.rs         # LRC timestamp parser
  helpers.rs              # Utility trait extensions
  websocket.rs            # Actix WebSocket actor
  player/                 # MPRIS2 player lifecycle
    mpris_loop.rs         # Top-level event loop
    player_discovery.rs   # D-Bus NameOwnerChanged listener
    player_observation.rs # Property change watcher
    lyric_session.rs      # Real-time lyric synchronization (SessionManager)
    mpris_metadata.rs     # Metadata extraction helpers
    dbus_proxies.rs       # zbus D-Bus proxy definitions
  lyric_cache.rs          # Disk lyric cache (read/write/cleanup)
  lyric_providers/        # Pluggable lyric sources
    mod.rs                # Provider registry, orchestration, fetch_netease_lyric helper
    file.rs               # Local tags (ID3/Vorbis) + LRC files
    mpris2_text.rs        # MPRIS2 Lyrics property fallback
    netease.rs            # NetEase Cloud Music search
    netease_trackid.rs    # NetEase via embedded track ID
    yesplaymusic.rs       # YesPlayMusic local API (port 10754)
    feeluown_netease.rs   # FeelUOwn integration
    splayer.rs            # SPlayer WebSocket (port 25885)
  web_routes/             # Actix HTTP handlers
    config.rs             # GET /config, POST /config/update
    test_page.rs          # Debug page at /test
frontend/
  kde/                    # Plasma 6 Plasmoid
    metadata.json         # Plugin manifest (version auto-injected by build script)
    contents/ui/          # QML files (main.qml, configBackend.qml, configFrontend.qml)
    config/               # KConfig XML schema
    translate/            # .po translation files (zh_CN, zh_TW)
  obs_studio/             # OBS Studio Python plugin
  build_kde_plasmoid.sh   # Builds .plasmoid package
docs/                     # User-facing documentation
assets/                   # Logos and screenshots
Cargo.toml / Cargo.lock   # Rust package manifest and lockfile
```

## Language and Toolchain

| Component | Language | Toolchain |
|-----------|----------|-----------|
| Backend | Rust 2024 edition | `cargo build --release` |
| Plasmoid | QML (Qt 6 / KDE Plasma 6) | bundled by `build_kde_plasmoid.sh` |
| OBS plugin | Python 3 | no build step |
| Build scripts | Bash | run directly |

Build the backend:
```sh
cargo build --release
```

Build the Plasma plasmoid package (`.plasmoid`):
```sh
cd frontend && bash build_kde_plasmoid.sh
```

## Architecture Notes

### Communication Flow

```
Music player (e.g. Elisa, Spotify)
  └─ D-Bus / MPRIS2
       └─ player/ module  →  lyric_providers/  →  state.rs
                                                      └─ broadcast channel
                                                           └─ websocket.rs  →  plasmoid / OBS
```

1. `mpris_loop` discovers the top-most MPRIS2 player.
2. `player_observation` watches `PropertiesChanged` on the player and fires metadata/seek/rate events.
3. `lyric_session::SessionManager` schedules wake-up timers based on LRC timestamps, playback position, and rate.
4. On each wake-up, the current lyric line (and optional translated line) is sent to `state.rs` which re-broadcasts via the Tokio broadcast channel.
5. Every connected WebSocket client (`websocket.rs`) receives the update.

### Lyric Provider Chain

Providers are tried in order; the first successful response wins. Order is defined in `lyric_providers/mod.rs`. Providers can be enabled/disabled at runtime via `Config.enabled_lyric_providers`.

Current providers (in default priority order):
1. `Mpris2Text` — MPRIS2 `xesam:asText` property
2. `File` — local ID3/Vorbis tags, then `~/Music/lrc/<title>.lrc`
3. `YesPlayMusic` — local YesPlayMusic API (localhost:10754)
4. `NeteaseTrackID` — embedded NetEase track ID in MPRIS metadata
5. `SPlayer` — SPlayer WebSocket (localhost:25885)
6. `FeelUOwnNetease` — FeelUOwn Python integration
7. `Netease` — online NetEase Cloud Music search

When adding a new provider, implement the `LyricProvider` async trait in a new file under `lyric_providers/`, register it in `mod.rs`, and add its identifier to the default `enabled_lyric_providers` list in `config.rs`.

### Lyric Cache

`lyric_cache.rs` provides a disk cache for NetEase lyrics keyed by numeric track ID. Cache files are stored as JSON at:

```
$LYRICA_CACHE_DIR/netease/<id>.json          # if LYRICA_CACHE_DIR is set
$XDG_CACHE_HOME/lyrica/netease/<id>.json     # else if XDG_CACHE_HOME is set
~/.cache/lyrica/netease/<id>.json            # fallback
```

Each file contains `{ lrc, tlyric, cached_at }`. The `LYRIC_CACHE` singleton is initialized lazily on first use.

Providers that call NetEase (`NeteaseTrackID`, `FeelUOwnNetease`, `Netease`) all go through the shared `fetch_netease_lyric(music_id, config)` function in `lyric_providers/mod.rs`, which handles cache lookup → network fetch → cache write in one place. This ensures the cache is shared across providers for the same track ID.

A background task in `main.rs` runs `cleanup_expired` at startup and then every 24 hours, deleting entries older than `lyric_cache_ttl_days`.

### WebSocket Protocol

Messages are JSON-encoded `ChannelMessage` variants (defined in `messages.rs`). Clients receive:
- `LyricText { text, translation }` — the lyric line to display
- `MusicInfo { title, artist, ... }` — song metadata on track change
- `ConfigUpdate { config }` — live config push

A new client immediately receives the current `State` on connection.

### HTTP API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/config` | Read current `Config` as JSON |
| `POST` | `/config/update` | Write and broadcast updated `Config` |
| `GET` | `/ws` | WebSocket upgrade |
| `GET` | `/test` | Debug page (dev only) |

## Configuration

Runtime config is stored in `~/.config/lyrica/config.json` (created on first run). Key fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `disabled_players` | `Vec<String>` | `["firefox", ...]` | MPRIS2 bus names to ignore |
| `enabled_lyric_providers` | `Vec<String>` | all providers | Active providers in order |
| `online_search_pattern` | `u8` | `0` | `0`=Title+Artist, `1`=Title only |
| `lyric_search_folder` | `String` | `~/Music/lrc` | Local LRC directory |
| `online_search_timeout_secs` | `u64` | `10` | HTTP timeout |
| `online_search_retry` | `bool` | `true` | Retry on failure |
| `online_search_max_retries` | `u32` | `3` | Max retry attempts |
| `lyric_cache_enabled` | `bool` | `true` | Enable disk lyric cache |
| `lyric_cache_ttl_days` | `u32` | `30` | Cache TTL in days; `0` = never expire |

## Coding Conventions

- **Async runtime:** single-threaded Tokio (`#[tokio::main(flavor = "current_thread")]`). Do not introduce multi-thread patterns.
- **Error handling:** functions inside the player/provider subsystems return `anyhow::Result`; WebSocket/HTTP handlers propagate Actix errors. Avoid `unwrap` on paths that can fail at runtime.
- **State sharing:** shared state is wrapped in `Arc<Mutex<State>>` and passed down via function parameters, not globals. New state fields go in `state.rs`.
- **Channel messages:** extend `ChannelMessage` in `messages.rs` when adding new event types; do not create separate channels.
- **Provider interface:** all providers implement `LyricProvider` from `lyric_providers/mod.rs`; do not call provider logic directly from the player module.
- **Config changes:** always broadcast a `ChannelMessage::ConfigUpdate` after mutating `Config` so all WebSocket clients stay in sync.
- **Comments:** only where the *why* is non-obvious — no explanatory prose for what the code already says.

## Logging

The project uses the `log` crate with `env_logger`. Set verbosity at runtime:

```sh
RUST_LOG=debug cargo run
RUST_LOG=lyrica=trace cargo run   # scope to this crate only
```

### Level conventions (derived from existing usage)

| Level | When to use | Examples in codebase |
|-------|------------|----------------------|
| `info!` | Discrete, meaningful state transitions that are always useful to see in normal operation, without being noisy. | Player added/removed (`player_discovery.rs`); lyric provider being tried (`lyric_providers/mod.rs`); playback session bootstrap with metadata and position (`player_observation.rs`); WebSocket client connect/disconnect (`websocket.rs`); config update received (`web_routes/config.rs`); playback events (seek, pause, rate change — but **not** poll) from `lyric_session.rs`; broadcast channel lag warnings (`state.rs`). |
| `debug!` | Intermediate values and conditional branches that are only needed when diagnosing a specific problem. Should not appear in a healthy normal run. | NetEase search result list and name/duration comparison logic (`netease.rs`); SPlayer WebSocket connection attempt, failure, and raw response frames (`splayer.rs`). |
| `trace!` | Fine-grained per-event noise: every property-change signal, every timer tick, every poll event. Currently unused in the codebase; reserve for future high-frequency paths (e.g. `PropertiesChanged` signal flooding, lyric timer ticks). |

### What not to log

- Do not log at `info` for events that fire continuously or on a fixed interval (e.g. `PlaybackEvent::Poll`) — use `debug` or `trace` instead.
- Do not log sensitive user data (file paths containing personal information, full lyric bodies at `info`).
- The commented-out `// info!("Property changed: ...")` in `player_observation.rs` is intentionally silenced because it fires on every D-Bus property update; if you re-enable it, downgrade to `trace!`.

## Versioning and Release

- Version is stored in `Cargo.toml` (`package.version`).
- `build_kde_plasmoid.sh` reads the Cargo version and injects it into `frontend/kde/metadata.json`.
- Release commits are tagged `v<major>.<minor>` (e.g. `v0.22`).
- When bumping the version, update `Cargo.toml` and re-run the build script; do not manually edit `metadata.json`.

## Testing

There is no automated test suite. Manual verification steps:

1. Start the daemon: `cargo run` (or install the release binary).
2. Open a music player supported by MPRIS2 and play a track.
3. Check the `/test` debug page (`http://127.0.0.1:15650/test`) to confirm WebSocket messages arrive.
4. Install the plasmoid and confirm lyric display on the Plasma panel.

When modifying lyric provider logic, test against a real music player with a known-lyric track and verify the correct provider fires (visible in `RUST_LOG=debug` output).

## Scope Limitations

- Lyrica is designed for standard Linux desktop installs. Do not add Flatpak/Snap/NixOS-specific paths or containerization logic.
- The backend binds only to `127.0.0.1`; do not expose it to the network.
- KDE Plasma 6.0+ is the only supported plasmoid target; do not backport to Plasma 5.
