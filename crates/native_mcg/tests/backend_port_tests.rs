use anyhow::Result;
use native_mcg::config::Config;
use native_mcg::BackendBuilder;
use std::net::TcpListener;

#[tokio::test]
async fn test_default_port_resolution() -> Result<()> {
    let cfg = Config::default();
    assert_eq!(cfg.port, 3000);
    assert!(!cfg.strict_port);

    let backend = BackendBuilder::new(cfg).build()?;
    assert_eq!(backend.bind_addr().ip().to_string(), "0.0.0.0");
    assert!(backend.bind_addr().port() >= 3000);
    Ok(())
}

#[tokio::test]
async fn test_port_builder_override() -> Result<()> {
    let cfg = Config::default();
    let backend = BackendBuilder::new(cfg).with_port(4500).build()?;
    assert_eq!(backend.bind_addr().ip().to_string(), "0.0.0.0");
    assert!(backend.bind_addr().port() >= 4500);
    Ok(())
}

#[tokio::test]
async fn test_dynamic_port_zero() -> Result<()> {
    let cfg = Config::default();
    let backend = BackendBuilder::new(cfg).with_port(0).build()?;
    assert_eq!(backend.bind_addr().port(), 0);
    Ok(())
}

#[tokio::test]
async fn test_explicit_bind_addr() -> Result<()> {
    let cfg = Config::default();
    let addr = "127.0.0.1:9876".parse()?;
    let backend = BackendBuilder::new(cfg).with_bind_addr(addr).build()?;
    assert_eq!(backend.bind_addr(), addr);
    Ok(())
}

#[tokio::test]
async fn test_occupied_port_non_strict_finds_next() -> Result<()> {
    // Occupy an ephemeral port
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let occupied_port = listener.local_addr()?.port();

    let cfg = Config {
        port: occupied_port,
        strict_port: false,
        ..Default::default()
    };

    let backend = BackendBuilder::new(cfg)
        .with_bind_host("127.0.0.1".parse::<std::net::IpAddr>()?)
        .build()?;
    assert_ne!(backend.bind_addr().port(), occupied_port);
    assert!(backend.bind_addr().port() > occupied_port);
    Ok(())
}

#[tokio::test]
async fn test_occupied_port_strict_fails() -> Result<()> {
    // Occupy an ephemeral port
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let occupied_port = listener.local_addr()?.port();

    let cfg = Config {
        port: occupied_port,
        strict_port: true,
        ..Default::default()
    };

    let result = BackendBuilder::new(cfg)
        .with_bind_host("127.0.0.1".parse::<std::net::IpAddr>()?)
        .build();
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("already in use") || err_msg.contains("strict mode enabled"));
    Ok(())
}
