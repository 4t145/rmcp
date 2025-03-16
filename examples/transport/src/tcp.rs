use common::caculater::Calculater;
use rmcp::{
    ClientHandlerService, ServerHandlerService, serve_client, serve_server, transport::io::async_rw,
};
mod common;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::spawn(server());
    client().await?;
    Ok(())
}

async fn server() -> anyhow::Result<()> {
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:8001").await?;
    while let Ok((stream, _)) = tcp_listener.accept().await {
        tokio::spawn(async move {
            let (rx, tx) = stream.into_split();
            let transport = async_rw(rx, tx);
            let service = ServerHandlerService::new(Calculater);
            let server = serve_server(service, transport).await?;
            server.waiting().await?;
            anyhow::Ok(())
        });
    }
    Ok(())
}

async fn client() -> anyhow::Result<()> {
    let stream = tokio::net::TcpSocket::new_v4()?
        .connect("127.0.0.1:8001".parse()?)
        .await?;
    let (rx, tx) = stream.into_split();
    let client = serve_client(ClientHandlerService::new(None), async_rw(rx, tx)).await?;
    let tools = client.peer().list_tools(Default::default()).await?;
    println!("{:?}", tools);
    Ok(())
}
