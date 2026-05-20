# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                  # build
cargo run                    # run (requires .env)
cargo test                   # run all tests
cargo test <test_name>       # run a single test (e.g. cargo test test_uses)
cargo clippy                 # lint
```

## Environment

Copy `.env.sample` to `.env` and fill in values before running:

| Variable | Description |
|---|---|
| `TELOXIDE_TOKEN` | Telegram bot token from BotFather |
| `AUTH_TOKEN` | Secret token used to authenticate the bot owner |
| `DB_PATH` | Path to SQLite database file (e.g. `./telebot.db`) |
| `TIMEOUT` | Shell command execution timeout in seconds |
| `SCHEDULE_CHAT_ID` | Chat ID that receives scheduled messages |
| `RUST_LOG` | Log level (e.g. `info`) |
| `OLLAMA_SERVER` | Ollama API base URL (default: `http://localhost:11434`) |
| `OLLAMA_MODEL` | Ollama model name (default: `qwen2.5:7b`) |

## Architecture

Single-owner Telegram bot: only one chat ID is authorized at a time. The first user to send `/auth <AUTH_TOKEN>` becomes the owner; all subsequent unauthorized users trigger a warning notification to the owner.

**State** (`states.rs`): `SqliteState` is the shared `Arc`-wrapped state injected into all handlers via `dptree`. It stores the authenticated `chat_id` in both SQLite (persistent) and an `AtomicI64` (in-memory cache). It also tracks a per-user `current_path` used as the working directory for shell commands.

**Message handler** (`msg_handles.rs`): Plain text messages from the authorized user are executed as bash commands in `current_path`. `cd <path>` is special-cased to change `current_path` (persisted to SQLite). Uploaded documents are saved to `current_path`. Command output is truncated by `TIMEOUT`.

**Command handler** (`cmd_handles.rs` + `cmd_handles/ollama_ops.rs`):
- `/start` — status check
- `/auth <token>` — authenticate and claim bot ownership
- `/down <filename>` — send a file from `current_path` back to the user
- `/chat <prompt>` — forward prompt to local Ollama and reply with the response

**Schedule loop** (`schedule_task.rs`): A separate `tokio::task` polls every 10 seconds and sends a message to `SCHEDULE_CHAT_ID`. Shutdown is coordinated via a `broadcast::channel` and `AtomicBool` shared with a SIGINT/SIGTERM signal thread.

**Deployment**: Runs as a `systemd` service (see `README.md` for unit file). Build with `cargo build --release` and copy the binary to the server.
