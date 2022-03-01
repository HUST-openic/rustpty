# Rust pty for xtermjs

Test rust pty scripts for web terminal backend.

Currently working on linux/macOS using *uix*. For windows, add [winpty-rs](https://crates.io/crates/winpty-rs).

After all, it's totally possible to use rust to spawn and interact with local shells. No websockets needed. What we have to do is to use rust pty crate to spawn local shell and get output. Then we can deliver this content to xtermjs using [tauri events](https://tauri.studio/docs/guides/events/).

TODO:

- Pass keyboard input to shell.
- Get shell output for terminal - GOODY!.
