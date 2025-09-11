# Iroh Rust Library Research for MCG P2P Card Game

## Overview
Iroh is a Rust library for establishing peer-to-peer (P2P) QUIC connections dialed by public key. It handles direct connectivity with hole-punching and falls back to relay servers for reliability. Built on QUIC (via Quinn), it provides authenticated, encrypted connections with low latency and features like concurrent streams. The library is part of the n0-computer project, licensed under MIT/Apache-2.0, and is actively maintained (latest version 0.91.2 as of Aug 2025).

Key repository: [n0-computer/iroh](https://github.com/n0-computer/iroh)  
Documentation: [iroh.computer/docs](https://iroh.computer/docs/)  
Rust Docs: [docs.rs/iroh](https://docs.rs/iroh/latest/iroh/)

## Core Features
- **Connection Establishment**: Uses `Endpoint` to bind and connect. Connections are initiated with a `NodeAddr` (combines `NodeId` (public key) and addressing info like direct IPs or `RelayUrl`). Supports ALPN for protocol negotiation.
- **Hole-Punching and Relays**: Attempts direct P2P via hole-punching; falls back to relays (e.g., n0 relays at relay.n0.computer). Relays forward encrypted traffic without decryption.
- **Encryption and Authentication**: End-to-end TLS encryption. Nodes identified by `PublicKey`/`NodeId`. No client-server distinction; mutual authentication.
- **Streams**: QUIC streams (uni/bi-directional) are lightweight, interleaved, and non-blocking. Lazy creation: streams activate only when data is sent.
- **Discovery**: Services like `DnsDiscovery` publish/lookup `NodeAddr` via DNS (e.g., dns.iroh.link). Enables connecting by `NodeId` alone.
- **Metrics and Monitoring**: Built-in metrics for performance; tools like `net_report` for network diagnostics.
- **Composability**: Supports pluggable protocols. Examples include:
  - `iroh-blobs`: Content-addressed blob transfer (BLAKE3 hashes) for sharing files/assets like card images.
  - `iroh-gossip`: Pub-sub overlay for broadcasting game events (e.g., turns, state updates).
  - `iroh-docs`: Eventually consistent KV store for game state sync.
- **WASM/Browser Support**: Works in browsers via WASM, aligning with MCG's frontend.
- **Other**: `RelayMap` for custom relays; `Watcher` for reactive updates; no head-of-line blocking.

Dependencies include `quinn`, `tokio`, `rustls`, and crypto libs like `ed25519-dalek`.

## Relevance to MCG P2P Card Game
MCG aims for P2P multiplayer (each player runs a backend node communicating directly). Iroh fits perfectly for low-latency, reliable connections without central servers. Key useful features:

- **Direct P2P with Fallback**: Enables fast, private games via hole-punching. Relays ensure connectivity behind NATs/firewalls—critical for mobile/web players. Reduces latency for real-time actions (e.g., card plays, betting).
- **QUIC Streams for Game Logic**: Use bi-directional streams for turn-based messaging (e.g., send card choice, receive opponent response). Uni-directional for broadcasts (e.g., game state updates). Interleaving supports concurrent actions like chat or asset loading without blocking.
- **Node Discovery**: Players share `NodeId` (public key) via QR/invite. DNS discovery allows easy reconnection without IPs. Integrates with MCG's QR scanner.
- **State Synchronization**: 
  - `iroh-gossip` for pub-sub: Broadcast game events (deals, folds) to all players efficiently.
  - `iroh-blobs` for sharing card images/assets or game history blobs.
  - `iroh-docs` for shared game state (e.g., pot, hands—encrypted per player view).
- **Security**: Built-in E2E encryption protects sensitive data (e.g., hidden cards). Public key auth prevents imposters.
- **Low Overhead**: Streams are cheap; suitable for resource-constrained devices (phones/browsers). QUIC's datagrams could handle unreliable UDP-like messages if needed (e.g., animations).
- **Backend Integration**: MCG's `native_mcg` already has partial iroh support (`iroh.rs`). Extend for full P2P: Replace WS with iroh streams; use `NodeAddr` for peer connections.
- **Challenges/Mitigations**:
  - Relay dependency: Use public n0 relays or self-host via `iroh-relay`.
  - Discovery: Implement custom service discovery if DNS insufficient (e.g., via game lobby).
  - WASM Limits: Iroh supports WASM, but test hole-punching in browsers.

Potential MCG Enhancements:
- P2P Game Rooms: Connect via `NodeId`, use streams for `ClientMsg`/`ServerMsg` protocol.
- Asset Sharing: Use `iroh-blobs` for dynamic card packs.
- Offline/Resume: Blobs for saving/loading game state.

## Integration Notes
- Add to `Cargo.toml`: `iroh = "0.91"`.
- Basic Setup (from docs):
  ```rust
  use iroh::{Endpoint, NodeAddr};
  let ep = Endpoint::builder().bind().await?;
  let conn = ep.connect(node_addr, b"mcg-protocol/1").await?;
  let (mut send, mut recv) = conn.open_bi().await?;
  // Send game message
  send.write_all(serde_json::to_string(&game_msg)?.as_bytes()).await?;
  ```
- For Accepting: Use `Router` to handle protocols.
- Existing MCG Code: Build on `native_mcg/src/backend/iroh.rs`—it already sends `Welcome` and `State` messages.
- Testing: Use `iroh-net-report` for NAT traversal checks. Examples in repo for echo, file transfer.
- Limitations: No built-in serialization (use Serde); relay costs if self-hosted.

## References
- [Iroh Quickstart](https://iroh.computer/docs/quickstart)
- [Endpoint Docs](https://docs.rs/iroh/latest/iroh/endpoint/index.html)
- [GitHub Examples](https://github.com/n0-computer/iroh/tree/main/iroh/examples)
- [Protocols](https://iroh.computer/proto)
- [Awesome Iroh](https://github.com/n0-computer/awesome-iroh)
- Changelog: Frequent updates; focus on stability and WASM.

This research is based on docs as of Sep 2025. Recommend testing a prototype integration in MCG's native backend.