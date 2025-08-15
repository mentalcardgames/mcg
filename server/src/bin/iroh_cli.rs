use anyhow::Result;
use iroh::endpoint::Endpoint;
use iroh_base::{NodeAddr, NodeId};
use mcg_shared::ClientMsg;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let peer = if let Some(p) = args.next() {
        p
    } else {
        eprintln!("Usage: iroh_cli <peer-node-id>");
        std::process::exit(1);
    };

    let node_id: NodeId = peer.parse().map_err(|e| anyhow::anyhow!("invalid node id: {}", e))?;
    let node_addr = NodeAddr::new(node_id);

    println!("iroh-cli: dialing peer {}", peer);

    let endpoint = Endpoint::builder().discovery_n0().bind().await?;
    let conn = endpoint.connect(node_addr, b"/mcg/msg/1").await?;
    let (mut send, mut _recv) = conn.open_bi().await?;

    let msg = ClientMsg::RequestState;
    let bytes = serde_json::to_vec(&msg)?;
    // Use the same framing implementation as the server transport module
    let framed = mcg_server::transport::framing::encode_frame(&bytes);

    use tokio::io::AsyncWriteExt;
    send.write_all(&framed).await?;
    // finish() is synchronous in this iroh version
    send.finish()?;

    println!("iroh-cli: sent RequestState to peer {}", peer);

    Ok(())
}
