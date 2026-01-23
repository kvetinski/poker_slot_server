# Poker Slot Server

Minimal Rust server that models a simple card game flow (deck → hand → discard) and exposes game logic over HTTP.

---

## Overview

* Rust + Axum HTTP server (async, Tokio)
* In-memory game state (`InMem`) for fast prototyping
* Core card-game lifecycle: deal → discard → evaluate → payout
* Dockerized for reproducible local/dev runs

---

## Features

* Standard 52-card deck
* Draw / discard mechanics
* Hand evaluation:

  * Pair, Two Pair, Trips
  * Straight, Flush
  * Full House, Quads
  * Straight Flush
* Simple in-memory store (no external DB)

---

## Quick Start

### Docker (recommended)

```bash
make up      # build & run
make logs    # follow logs
make down    # stop containers
make clean   # remove containers & volumes (project-only)
```

Server runs on: `http://0.0.0.0:3001`

### Cargo (local dev)

```bash
cargo run --release
```

---

## Project Structure

```
src/
  main.rs        # app bootstrap, layers, server start
  lib.rs
  server/        # router + HTTP handlers
  store/         # InMem store, shared state
  middleware/    # logging, CORS
```

---

## API

Routes are defined in `server::router`. See source for exact endpoints and payloads.

---

## Notes

* Designed for clarity and iteration, not persistence
* Replace `InMem` with a DB for production
* Tighten CORS and logging before deployment

---

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
