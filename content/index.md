+++
title = "pmacs"
booth = "For Linux and macOS. The GPU frontend draws with Vulkan on Linux and Metal on macOS, where the build is an unsigned beta. The terminal frontend attaches locally or over SSH."
+++

pmacs runs the editor's hot path (rope, buffers, views, async runtime,
process supervision) in Rust, and exposes the rest --- commands, keymaps,
hooks, packages --- through an embedded Lua VM. The design follows Emacs
in shape (configurable, introspectable, programmable from inside) but
discards the single-threaded substrate; workers, message bus, and a
coroutine-based async surface are core primitives, not bolt-ons.

[Two frontends, one editor](/architecture/) describes how the pieces fit.
