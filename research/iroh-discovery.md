# Iroh Discovery Mechanisms

## Overview
Discovery resolves `NodeId` to `NodeAddr` (RelayUrl + direct addresses) for connections. Multiple services can combine via `ConcurrentDiscovery`. Essential for MCG: Players share NodeId via QR; auto-resolve for P2P.

## Key Traits
- **Discovery**: Core trait.
  - `resolve(node_id: NodeId) -> impl Future<Output=Result<Vec<NodeAddr>>>`: Lookup addresses.
  - `subscribe() -> impl Stream<Item=DiscoveryItem>`: Watch for new nodes (passive discovery).
  - `publish(node_info: NodeInfo) -> impl Future<Output=()>`: Publish local info.
- **IntoDiscovery**: Builder for discovery (gets `DiscoveryContext` with secret key/DNS).

`NodeInfo`: `NodeId` + `NodeData` (direct addrs, user data).  
`DiscoveryItem`: New/updated `NodeInfo` + source.

## Implementations
1. **DnsDiscovery**: DNS-based (TXT records via pkarr.org).
   ```rust
   use iroh::discovery::dns::DnsDiscovery;
   let discovery = DnsDiscovery::n0();  // Public n0 server
   endpoint.builder().add_discovery(discovery);
   ```
   - Resolve: Queries `_nodeid.pkarr.iroh.link`.
   - Publish: Via `PkarrPublisher` (separate service).

2. **PkarrPublisher**: Publishes to pkarr relays (HTTP/DHT).
   ```rust
   use iroh::discovery::pkarr::PkarrPublisher;
   let publisher = PkarrPublisher::n0();  // Or custom relay
   endpoint.builder().add_discovery(publisher);
   ```
   - Auto-publishes on bind; republishes on changes.
   - UserData: Optional string (e.g., game metadata).

3. **MdnsDiscovery**: Local network via mDNS (requires `discovery-local-network` feature).
   ```rust
   use iroh::discovery::mdns::MdnsDiscovery;
   let mdns = MdnsDiscovery::builder().spawn();
   endpoint.builder().add_discovery(mdns);
   ```
   - Discovers peers on LAN; useful for local multiplayer.

4. **StaticProvider**: Manual addresses.
   ```rust
   use iroh::discovery::static_provider::StaticProvider;
   let mut static_prov = StaticProvider::default();
   static_prov.insert(node_id, node_addr);
   endpoint.builder().add_discovery(static_prov);
   ```

5. **ConcurrentDiscovery**: Combine (e.g., DNS + MDNS).
   ```rust
   let concurrent = ConcurrentDiscovery::new(vec![dns, mdns]);
   ```

## Usage in Endpoint
```rust
let endpoint = Endpoint::builder()
    .add_discovery(PkarrPublisher::n0())
    .add_discovery(DnsDiscovery::n0())
    .add_discovery(MdnsDiscovery::default())
    .bind()
    .await?;

// Auto-resolve on connect
let conn = endpoint.connect(their_node_id, b"mcg").await?;

// Watch discoveries
let mut stream = endpoint.discovery_stream();
while let Some(item) = stream.next().await {
    match item {
        Ok(DiscoveryItem::New(node_info)) => { /* New peer */ }
        Ok(DiscoveryItem::Update(node_info)) => { /* Addr update */ }
        Err(Lagged) => { /* Missed events */ }
    }
}
```

## For MCG
- Use DNS/Pkarr for global (share NodeId via QR).
- MDNS for local play.
- Publish game-specific user data (e.g., room ID).
- On connect fail: Fallback to manual `add_node_addr` from invite.

Errors: `DiscoveryError` (NotFound, Timeout), `IntoDiscoveryError` (build fails).