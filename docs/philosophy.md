# Philosophy: A Human Window into AI-Piloted Work

## Core Purpose

Beads Task-Issue Tracker is **a human control panel** for an AI-native issue tracker.

The Beads CLI (`br` or `bd`) is designed for AI agents — they create issues, update statuses, and pilot workflows programmatically. This application gives humans the visibility and controls to observe what the AI is driving, and to step in when needed.

## Following the CLI, Not Leading It

This application reads what the CLI writes and presents it for humans. It does not define the issue format, the database schema, or the sync protocol.

When the CLI evolves, we adapt. When it adds features, we surface them. But if the CLI becomes purely machine-to-machine with no human-interpretable output, we freeze at the last meaningful version. The goal is human readability.

## Why br Is Our Recommended CLI

The original Beads philosophy was simple: issues stored as JSONL, backed by SQLite, with a lightweight CLI that writes human-readable files. No daemon, no server, no complex infrastructure.

The `bd` CLI (Go) has progressively drifted from this — Dolt migration, server mode, daemon lifecycle management. Each step added complexity that works against standalone desktop use. A server that must be started, managed, and stopped is the opposite of "open a folder and see your issues."

[`br`](https://github.com/Dicklesworthstone/beads_rust) (beads_rust) froze at the classic architecture: SQLite + JSONL, no daemon, no server. It is faster, more predictable, and aligned with our core principle — **a human should be able to open a project and immediately see what's happening**.

We support both:

- **`br`** (recommended) — fast, stable, aligned with the original philosophy
- **`bd` 0.49.x** — last Go version before server mode, fully supported as fallback

If `bd` restores simple, file-based operation with native change notifications, we will reconsider. Until then, `br` is the path that best serves this application's purpose.

## Design Principle: Observe and Edit

Two complementary roles:

1. **Observe** — dashboards, issue lists, change detection
2. **Edit** — create, update, comment, reassign, close

The AI pilots through the CLI, the human monitors and corrects through this app. This is a control panel, not a workspace — lightweight, fast to open, immediately useful.

---

*Laurent Chapin*
