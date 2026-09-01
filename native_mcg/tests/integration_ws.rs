use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig, PlayerId};
use std::time::Duration;

#[tokio::test]
async fn ws_broadcasts_state_to_other_clients() -> Result<()> {
    // Start an axum server on an OS-assigned port using the same router as the binary.
    let app = native_mcg::server::run::build_router(native_mcg::config::Config::default(), None);

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

    // Connect two websocket clients.
    let (ws1_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (ws2_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;

    let (mut write1, mut read1) = ws1_stream.split();
    let (_write2, mut read2) = ws2_stream.split();

    // Direct responses (e.g. Ping) are routed only to the originating connection.
    write1
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&Frontend2BackendMsg::Ping)?,
        ))
        .await?;
    let pong = tokio::time::timeout(Duration::from_secs(1), read1.next())
        .await?
        .expect("requesting websocket should remain open")?;
    let tokio_tungstenite::tungstenite::Message::Text(pong) = pong else {
        panic!("expected pong as text");
    };
    assert!(matches!(
        serde_json::from_str::<Backend2FrontendMsg>(&pong)?,
        Backend2FrontendMsg::Pong
    ));
    assert!(
        tokio::time::timeout(Duration::from_millis(200), read2.next())
            .await
            .is_err(),
        "other clients should not receive direct request responses"
    );

    // Client 1 sends NewGame which should trigger a broadcasted State to all connected clients
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

    let cm = Frontend2BackendMsg::NewGame { players };
    let txt = serde_json::to_string(&cm)?;
    write1
        .send(tokio_tungstenite::tungstenite::Message::Text(txt))
        .await?;

    // Now assert client 2 receives a State message within a short timeout
    let mut got_state = false;
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt)))) =
            tokio::time::timeout(Duration::from_millis(300), read2.next()).await
        {
            if let Ok(Backend2FrontendMsg::UpdatePokerState(_)) =
                serde_json::from_str::<Backend2FrontendMsg>(&txt)
            {
                got_state = true;
                break;
            }
        }
    }

    assert!(
        got_state,
        "client2 did not receive a State after client1 NewGame"
    );

    // Clean up server
    server_handle.abort();
    Ok(())
}
