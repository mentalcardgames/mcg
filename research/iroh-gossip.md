# Iroh-Gossip: Pub-Sub for P2P Overlays

## Overview
`iroh-gossip` implements epidemic broadcast trees (HyParView + PlumTree) for scalable pub-sub over iroh connections. Ideal for MCG: Broadcast game events (e.g., card deals, turns) to peers without central server. Topics partition swarms; messages flood via tree (eager push) + anti-entropy (lazy pull).

Repo: [n0-computer/iroh-gossip](https://github.com/n0-computer/iroh-gossip) (v0.31.0).  
Add: `iroh-gossip = "0.31"`. Depends on `iroh`.

## Core Concepts
- **Topic**: String identifier for swarm (e.g., "mcg-game-abc123"). Peers join/leave topics.
- **Swarm**: Peers subscribed to a topic form a dynamic overlay.
- **Gossip**: Messages (up to 1KB) flooded reliably. Uses iroh streams for transport.
- **View**: Each peer maintains active (tree neighbors) and passive (candidates) views.
- **IHave/IWant**: Anti-entropy for missed messages (Bloom filters track digests).

## Protocol Modules
- **Proto**: State machine for gossip logic (no IO). `GossipProto` handles join/prune, message relay.
- **Net**: Networking over iroh. `Gossip` spawns on `Router` with ALPN `b"iroh-gossip/0"`.

## Setup and Usage
Integrate with iroh `Router`:

```rust
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{net::Gossip, proto::GossipProto, Topic};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::builder().bind().await?;
    let topic = Topic::new("mcg-game");  // Or derive from game ID

    // Create gossip instance
    let mut gossip = Gossip::builder(endpoint.clone())
        .spawn(topic.clone())?;

    // Join swarm
    gossip.join().await?;

    // Publish message
    let msg_id = gossip.publish(vec![1u8; 100]).await?;  // Returns MsgId (BLAKE3 hash)

    // Subscribe to messages
    let mut events = gossip.subscribe();
    while let Some(event) = events.recv().await {
        match event {
            iroh_gossip::net::Event::Message(msg) => {
                if msg.id() == msg_id { /* Handle game event */ }
            }
            iroh_gossip::net::Event::Joined => println!("Joined swarm"),
            _ => {}
        }
    }

    // Leave
    gossip.leave().await?;
    Ok(())
}
```

- **Gossip::builder()**: Configures max views, etc.
- **Gossip::spawn(topic) -> Result<Gossip>**: Starts protocol.
- **join()**: Connect to random peers via discovery; build overlay.
- **publish(payload: Vec<u8>) -> Result<MsgId>`: Flood message to topic. Payload <1KB.
- **subscribe() -> mpsc::Receiver<Event>`: Receive messages/joins/leaves.
- Events: `Message { id, payload, sender: NodeId }`, `Joined`, `Left`, `Pruned`.

For MCG:
- Topic per game room (e.g., hash of game ID).
- Publish: `publish(serde_json::to_vec(&GameEvent::Deal { cards })?)`.
- Subscribe: Deserialize events, update state (e.g., via egui).
- Scale: Handles 100s of peers; low overhead (O(log N) per message).

## Advanced
- **Custom Transport**: Implement `Transport` trait for non-iroh backends.
- **Metrics**: Counters for messages sent/received, view sizes.
- **Config**: `GossipConfig { active_view_size: 5, passive_view_size: 20, ... }`.

## For MCG
- Replace WS broadcasts with gossip for P2P events (bets, folds).
- Use with iroh-blobs for card assets.
- Discovery: Auto-join via DNS/MDNS for local/multiplayer.

See [Gossip Chat Example](https://iroh.computer/docs/examples/gossip-chat).