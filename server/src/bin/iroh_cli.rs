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
        eprintln!("Usage: iroh_cli <peer-node-id-or-nodeaddr>");
        std::process::exit(1);
    };

    println!("iroh-cli: dialing {}", peer);

    let endpoint = Endpoint::builder()
        .alpns(vec![b"/mcg/msg/1".to_vec()])
        .discovery_n0()
        .bind()
        .await?;

    // Try to parse the peer string as a NodeTicket (ticket string). If that fails,
    // fall back to parsing as a bare NodeId and construct an ID-only NodeAddr.
    let target_addr: NodeAddr = match peer.parse::<iroh_base::ticket::NodeTicket>() {
        Ok(ticket) => ticket.into(),
        Err(_) => {
            // try NodeId
            match peer.parse::<NodeId>() {
                Ok(nid) => NodeAddr::new(nid),
                Err(e) => {
                    eprintln!("Invalid peer identifier: not a NodeTicket or NodeId: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    // Local implementation of the same length-prefixed framing (u32 BE)
    fn encode_frame(payload: &[u8]) -> Vec<u8> {
        use bytes::BufMut;
        let mut buf = Vec::with_capacity(4 + payload.len());
        buf.put_u32(payload.len() as u32);
        buf.extend_from_slice(payload);
        buf
    }

    // Retry loop to tolerate discovery/relay propagation delays.
    let max_attempts = 6;
    let mut attempt = 0usize;
    loop {
        attempt += 1;
        match endpoint.connect(target_addr.clone(), b"/mcg/msg/1").await {
            Ok(conn) => {
                let (mut send, mut _recv) = conn.open_bi().await?;
                let msg = ClientMsg::RequestState;
                let bytes = serde_json::to_vec(&msg)?;
                let framed = encode_frame(&bytes);
                tokio::io::AsyncWriteExt::write_all(&mut send, &framed).await?;
                // finish() is synchronous in this iroh version
                send.finish()?;
                println!("iroh-cli: sent RequestState to peer {}", peer);
                return Ok(());
            }
            Err(e) => {
                eprintln!("iroh-cli: connect attempt {} failed: {:?}", attempt, e);
                if attempt >= max_attempts {
                    return Err(anyhow::anyhow!("Failed to connect after {} attempts: {:?}", attempt, e));
                }
                // Wait a bit before retrying to allow discovery/relay propagation
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        }
    }
}
