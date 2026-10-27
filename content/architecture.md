+++
title = "Two frontends, one editor"
description = "How pmacs is put together: a long-lived instance and the frontends that attach to it."
eyebrow = "About"
sign = "Gemini"
note = "Gemini is this page's sign: Castor and Pollux, two figures over one ground."
+++

The editor is partitioned into a long-lived instance (the daemon that owns
buffers, processes, and language services) and thin frontends that attach
over a typed, versioned protocol.

Two frontends ship today. A TUI, a crossterm cell grid, attaches locally
over a Unix socket or remotely over SSH, with reconnect-on-drop modeled on
`mosh`. pmacs-gpu, a GPU frontend (wgpu, winit and glyphon), renders from a
semantic projection of editor state (style spans, decorations, inlay
adornments) rather than a character grid, and edits optimistically against
a local CRDT replica for latency-free typing.

Buffers are CRDT-backed, so multiple frontends, TUI and GPU, local and
remote, can edit the same buffers concurrently with live cursor and
selection presence.
