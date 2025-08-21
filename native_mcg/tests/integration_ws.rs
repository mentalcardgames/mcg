use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use mcg_shared::{ClientMsg, PlayerConfig, PlayerId, ServerMsg};
use std::time::Duration;

#[tokio::test]
async fn ws_broadcasts_state_to_other_clients() -> Result<()> {
    // Start an axum server on an OS-assigned port using the same router as the binary.
    let state = native_mcg::backend::AppState::default();
    let app = native_mcg::backend::run::build_router(state.clone());

    // Bind to port 0 so the OS chooses an available port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // Spawn the server in background
    let server_handle = tokio::spawn(async move {
        let result = axum::serve(listener, app).await;
        if let Err(e) = result {
            eprintln!("server error: {}", e);
        }
    });

    // Give server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ws_url = format!("ws://127.0.0.1:{}/ws", addr.port());

    // Connect two websocket clients
    let (ws1_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (ws2_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;

    let (mut write1, mut read1) = ws1_stream.split();
    let (mut _write2, mut read2) = ws2_stream.split();

    // Drain initial welcome/state messages from both clients
    async fn drain_welcome_and_state<R>(read: &mut R) -> Option<ServerMsg>
    where
        R: StreamExt<Item = Result<tokio_tungstenite::tungstenite::Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        let mut last_state: Option<ServerMsg> = None;
        let start = tokio::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(Some(Ok(msg))) = tokio::time::timeout(Duration::from_millis(200), read.next()).await {
                if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                    if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                        if let ServerMsg::State(_) = &sm {
                            last_state = Some(sm);
                            break;
                        }
                        // record other msgs (e.g., Welcome)
                        last_state = Some(sm);
                    }
                }
            } else {
                // timeout or stream ended
                break;
            }
        }
        last_state
    }

    let _ = drain_welcome_and_state(&mut read1).await;
    let _ = drain_welcome_and_state(&mut read2).await;

    // Client 1 sends NewGame which should trigger a broadcasted State to client 2
    let players = vec![
        PlayerConfig {
            id: PlayerId(0),
            name: "Alice".to_string(),
            is_bot: false,
        },
        PlayerConfig {
            id: PlayerId(1),
            name: "Bob".to_string(),
            is_bot: true,
        },
    ];

    let cm = ClientMsg::NewGame { players };
    let txt = serde_json::to_string(&cm)?;
    write1.send(tokio_tungstenite::tungstenite::Message::Text(txt)).await?;

    // Now assert client 2 receives a State message within a short timeout
    let mut got_state = false;
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(Ok(msg))) = tokio::time::timeout(Duration::from_millis(300), read2.next()).await {
            if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                    if let ServerMsg::State(_) = sm {
                        got_state = true;
                        break;
                    }
                }
            }
        }
    }

    // Clean up server
    server_handle.abort();

    assert!(got_state, "client2 did not receive a State after client1 NewGame");
    Ok(())
}
