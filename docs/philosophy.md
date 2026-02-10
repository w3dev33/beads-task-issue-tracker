# Philosophy: A Human Window into AI-Piloted Work

## Core Purpose

Beads Task-Issue Tracker is, at its core, **a human interface** for an AI-native issue tracker.

The `bd` CLI was designed for AI agents — they create issues, update statuses, close tasks, and manage workflows programmatically. But humans need visibility and control over what's happening. This application exists to give humans both: a way to observe what the AI is piloting, and to step in and edit, correct, or redirect when needed.

## Why This Matters

As AI agents take on more engineering tasks, the question shifts from "how do I manage my issues?" to **"how do I oversee what the AI is piloting, and how do I take the wheel when I need to?"**

Traditional issue trackers (Jira, GitHub Issues) were built for humans to communicate with other humans. Beads was built for AI agents to pilot issue workflows. This application bridges the gap — it lets humans see what the AI is driving, and gives them the controls to intervene at any point.

## Following bd, Not Leading It

This application follows the `bd` CLI. It does not define the issue format, the database schema, or the sync protocol. It reads what `bd` writes and presents it in a human-friendly way.

This means:

- When `bd` evolves, we adapt
- When `bd` adds features, we surface them
- When `bd` changes its internals (daemon removal, Dolt migration), we detect and accommodate

However, there is a limit. If `bd` evolves to the point where its interface is purely machine-to-machine with no human-interpretable output, this application will freeze at the last version that remains meaningful to display. The goal is human readability — if the source data stops being human-readable, there's nothing left to visualize.

## The br Exception

Some users preferred the earlier, simpler architecture of Beads: SQLite + JSONL, no daemon, no Dolt. The `br` (beads_rust) client is a Rust port that froze at that classic architecture.

We added support for `br` as a one-time accommodation. It was a specific request from a user who found the evolving `bd` architecture harder to follow and preferred the stability of a frozen implementation.

This is not a precedent for supporting arbitrary clients. The application supports exactly two CLI backends:

- **`bd`** — the canonical Beads CLI, actively evolving
- **`br`** — the Rust port, frozen at the classic SQLite + JSONL architecture

No additional clients will be added unless they are fully compatible with the `bd` command interface and output format. The version detection system (`detect_cli_client`) is designed for these two clients specifically, with explicit feature profiles for each.

## Design Principle: Observe and Edit

This application serves two complementary roles:

1. **Observe** what the AI is piloting — dashboards, issue lists, change detection
2. **Edit** when the human needs to step in — create, update, comment, reassign, close

The workflow is collaborative: the AI pilots the issue lifecycle through `bd`, and the human uses this application to monitor progress and make corrections. Neither side works in isolation.

- Dashboard: What's the state of the project?
- Issue list: What has the AI been piloting?
- Issue details: What was written, what decisions were made?
- Edit capabilities: Correct a priority, reassign, add context, close prematurely
- Change detection: Has something changed since I last looked?

The moment this application tries to become a full-featured project management tool is the moment it loses focus. It should remain lightweight, fast to open, and immediately useful — a control panel, not a workspace.

---

*Laurent Chapin*
