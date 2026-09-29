use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig, PlayerId};
use std::time::Duration;

#[tokio::test]
async fn ws_broadcasts_state_to_other_clients() -> Result<()> {
    let (backend_handle, _tasks) =
        native_mcg::BackendBuilder::new(native_mcg::config::Config::default()).spawn()?;
    let addr = backend_handle
        .network()
        .start_axum_listener("127.0.0.1:0".parse()?)
        .await?;

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
    backend_handle.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn ws_broadcasts_player_setup_to_other_clients() -> Result<()> {
    let (backend_handle, _tasks) =
        native_mcg::BackendBuilder::new(native_mcg::config::Config::default()).spawn()?;
    let addr = backend_handle
        .network()
        .start_axum_listener("127.0.0.1:0".parse()?)
        .await?;

    let ws_url = format!("ws://127.0.0.1:{}/ws", addr.port());

    // Connect two websocket clients.
    let (ws1_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (ws2_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;

    let (mut write1, _read1) = ws1_stream.split();
    let (_write2, mut read2) = ws2_stream.split();

    let players = vec![
        PlayerConfig {
            id: PlayerId(0),
            name: "Alice".to_string(),
            is_bot: false,
        },
        PlayerConfig {
            id: PlayerId(1),
            name: "Bob".to_string(),
            is_bot: false,
        },
    ];

    let update_msg = Frontend2BackendMsg::UpdatePlayerSetup(players.clone());
    let txt = serde_json::to_string(&update_msg)?;
    write1
        .send(tokio_tungstenite::tungstenite::Message::Text(txt))
        .await?;

    // Client 2 should receive the PlayerSetup broadcast
    let mut received_setup = None;
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt)))) =
            tokio::time::timeout(Duration::from_millis(300), read2.next()).await
        {
            if let Ok(Backend2FrontendMsg::PlayerSetup(p)) =
                serde_json::from_str::<Backend2FrontendMsg>(&txt)
            {
                received_setup = Some(p);
                break;
            }
        }
    }

    let received = received_setup.expect("client 2 should receive PlayerSetup broadcast");
    assert_eq!(received.len(), 2);
    assert_eq!(received[0].name, "Alice");
    assert_eq!(received[1].name, "Bob");

    // Client 3 connects after setup was updated; should receive PlayerSetup upon connecting
    let (ws3_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (_write3, mut read3) = ws3_stream.split();

    let mut client3_setup = None;
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt)))) =
            tokio::time::timeout(Duration::from_millis(300), read3.next()).await
        {
            if let Ok(Backend2FrontendMsg::PlayerSetup(p)) =
                serde_json::from_str::<Backend2FrontendMsg>(&txt)
            {
                client3_setup = Some(p);
                break;
            }
        }
    }

    let received3 = client3_setup.expect("client 3 should receive PlayerSetup on connect");
    assert_eq!(received3.len(), 2);
    assert_eq!(received3[0].name, "Alice");
    assert_eq!(received3[1].name, "Bob");

    backend_handle.shutdown().await;
    Ok(())
}
