# Argus

A small Rust tool for the everyday mess of local development.

You know the feeling: Redis is up, Postgres is up, three `node` processes are running, something grabbed port `8080`, and `htop` is somehow both too much and not enough. Argus is meant for that moment — a focused look at **ports**, **dev processes**, and **Docker**, plus a simple TUI when you want to poke around interactively.

---

## What it does

- Shows which processes (and Docker containers) are listening on which ports
- Lists “dev-looking” processes with CPU and memory
- Shows Docker containers with CPU / MEM, and lets you stop / restart / remove them
- Works as a CLI for scripting (`--format json`) or as a live TUI dashboard

Right now it works best on **Linux and WSL**. macOS / Windows are on the radar.

---

## Getting started

You need a recent Rust toolchain. Docker is optional, but without it you lose container names on ports and the Docker tab.

```bash
git clone <your-repo-url> argus
cd argus
cargo build --release
```

The binary ends up at `target/release/argus`. For day-to-day hacking, this is fine too:

```bash
cargo run -- tui
```

---

## Everyday commands

```bash
# Open the dashboard
cargo run -- tui        # same as: cargo run -- ui

# Who is on 8080?
cargo run -- ports --port 8080

# Only the noisy local-dev stuff
cargo run -- ps --dev-only

# Quick RAM / CPU snapshot
cargo run -- stats

# Pipe into jq / scripts
cargo run -- ports --format json
cargo run -- ps -d --format json
```

---

## CLI

```text
argus tui      Interactive TUI          (alias: ui)
argus ports    Listening ports          (aliases: p, port)
argus ps       Processes                (alias: proc)
argus stats    System RAM / CPU         (aliases: st, sys)
```

### ports

```bash
argus ports
argus ports -p 6379
argus ports --format json
```

Each row is a listening socket. The **OWNER** column is the useful bit: either a process name, or something like `redis-dev (docker)` when Docker published that host port.

### ps

```bash
argus ps
argus ps -d              # --dev-only
argus ps --format json
```

Without `-d` you get a broader process list. With `-d`, Argus tries to keep only things that look like local development (Node, Python, Cargo, local DBs, and similar).

### stats

```bash
argus stats
argus stats --format json
```

Host memory and CPU. On WSL, the total RAM is whatever the WSL2 VM is allowed to use — not always the full Windows machine.

### tui

```bash
argus tui
```

Full-screen view. Data refreshes about every two seconds (press `r` to refresh now).

---

## Using the TUI

Three tabs:

1. **Ports** — look up who owns a port (read-only)
2. **Processes** — your local servers / tooling; press `x` to kill the selected process
3. **Docker** — containers with CPU / MEM; stop, restart, or remove

### Keys you’ll actually use

| Key | What it does |
|-----|----------------|
| `q` / `Esc` | Quit |
| `r` | Refresh |
| `1` `2` `3` | Jump to a tab |
| `Tab` | Cycle tabs |
| `↑` `↓` or `k` `j` | Move around |
| `/` | Search and jump to a match |
| `n` / `N` | Next / previous match |
| `x` | Kill process (Processes tab) |
| `s` | Stop container (Docker tab) |
| `S` | Restart container (Docker tab) |
| `d` | Remove container — then `y` to confirm, `n` / `Esc` to cancel |

Search is intentionally simple: type a query, hit Enter, and the selection jumps to the first hit. It does not hide the rest of the list.

---

## Ports + Docker

If Docker is running, Argus connects the dots for you:

```text
6379  →  redis-dev (docker)
5432  →  postgres-dev (docker)
8080  →  node
```

No Docker socket? Fine. Ports and processes still work; the Docker tab just tells you it couldn’t connect instead of taking the whole app down.

Seeing `::` in the ADDRESS column is normal — it usually means “listening on all interfaces,” which is common for published container ports.

---

## Project layout

Nothing fancy:

```text
collectors/   read from the OS and Docker
actions/      kill processes, stop/restart/remove containers
output/       CLI tables and JSON
tui/          the interactive UI
models.rs     shared types
```

Reading and changing things stay separate on purpose. Collectors don’t kill anything; the UI doesn’t talk to Docker directly for stop/remove.

### Libraries

Built with clap, ratatui, crossterm, sysinfo, netstat2, bollard, comfy-table, and serde.

---

## Things worth knowing

- Killing processes or talking to Docker needs the usual permissions. If something fails, the footer should say so.
- Docker CPU % is “share of the host,” same idea as `docker stats` — not “percent of the container’s CPU limit.”
- Linux / WSL first. Broader platform support comes later.

---

## Roadmap

Ideas, not promises — subject to change as the tool grows.

- **Richer Docker stats** — go beyond the current CPU / MEM columns. Volumes, disk usage, memory details (limits vs usage), and whatever else turns out useful without cluttering the main table. Details still TBD.
- Better macOS / Windows support
- Jump from a port row straight to its process or container
- Config for refresh interval and ignored system ports
- CLI helpers for kill / docker stop (same actions as the TUI)

---

## License

MIT — see [LICENSE](LICENSE).

---

Named after Argus Panoptes — the all-seeing one. Fitting for a tool that stares at your ports.
