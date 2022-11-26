use tokio::net::TcpListener;
use tokio::signal;
use test_tokio::server;

#[tokio::main]
pub async fn main() -> test_tokio::Result<()> {
    // Bind a TCP listener
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", "10086")).await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}