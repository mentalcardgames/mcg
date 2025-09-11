# Iroh-Blobs: Content-Addressed Blob Transfer

## Overview
`iroh-blobs` enables efficient P2P transfer of blobs (byte sequences) and sequences, verified by BLAKE3 hashes. Suited for MCG: Share card images, game states, or decks (small blobs) P2P. Supports partial ranges, resumption, and multi-provider requests.

Repo: [n0-computer/iroh-blobs](https://github.com/n0-computer/iroh-blobs) (v0.91.0).  
Add: `iroh-blobs = { version = "0.91", features = ["memstore"] }`. Uses iroh for transport.

## Core Concepts
- **Blob**: Arbitrary bytes; identified by 32-byte BLAKE3 hash (`Hash`).
- **HashSeq**: Sequence of hashes (multiple of 32 bytes); points to ordered blobs.
- **Ticket**: `Hash` + optional `HashSeq` + provider `NodeId` for easy sharing.
- **Store**: Local storage (e.g., `MemStore`, `fs::FsStore`). Manages add/get.
- **Provider**: Serves blobs via iroh streams (ALPN `b"iroh-blobs/1"`).
- **Requester**: Requests from providers; supports multi-provider for redundancy.

Protocol: Requester sends `Request` (hashes + ranges) over QUIC bi-stream; provider streams verified data.

## Setup and Usage
Integrate with iroh:

```rust
use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{store::mem::MemStore, Blobs, Tag};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::builder().bind().await?;
    let store = MemStore::default();  // Or FsStore::new("/path")
    let blobs = Blobs::builder(endpoint.clone())
        .store(Box::new(store))
        .spawn()?;

    // Add blob (e.g., card image)
    let tag = blobs.add_bytes(b"ace of spades PNG data...").await?;
    let ticket = blobs.ticket(tag).await?;  // Shareable: hash + provider NodeId

    // Provider auto-handles requests via Router
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn();

    // Request from ticket (remote)
    let remote_blobs = Blobs::from_ticket(endpoint, ticket.clone()).await?;
    let data = remote_blobs.get(tag).await?;  // Downloads to local store

    Ok(())
}
```

- **Blobs::builder()**: Configures store, endpoint.
- **add_bytes(data: &[u8]) -> Result<Tag>`: Add to store, get `Tag` (local hash).
- **ticket(tag: Tag) -> Result<Ticket>`: Create shareable ticket.
- **get(tag: Tag) -> Result<Vec<u8>>`: Download blob (verifies hash).
- **get_seq(seq: &[Hash]) -> Result<Vec<(Hash, Vec<u8>)>>`: Download sequence.
- **provider() -> Provider**: For serving: `provider.serve_request(request)`.
- **from_ticket(endpoint: Endpoint, ticket: Ticket) -> Result<Blobs>`: Remote accessor.

Ranges: `get_range(hash, start..end)` for partial downloads (e.g., large decks).

Stores:
- `MemStore`: In-memory (default).
- `FsStore`: Disk-based, with hashing on add.

## Advanced
- **Multi-Provider**: `Download` with multiple `NodeId`s; parallel fetches.
- **Partial Sync**: Request specific ranges/sequences for delta updates.
- **Verification**: Streams include BLAKE3 chunks; rejects tampered data.
- **Resumption**: Requests include offsets; supports interruptions.

## For MCG
- Store card PNGs as blobs; share tickets via QR for P2P asset sync.
- Game state: Serialize to blob, share hash for verification.
- Integrate: In `native_mcg`, use `Blobs` for media/alt_cards sharing.
- With Gossip: Announce new blobs via pub-sub.

See [Sendme CLI Example](https://github.com/n0-computer/sendme) for file sharing.