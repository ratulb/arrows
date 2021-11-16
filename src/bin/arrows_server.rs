use async_std::net::TcpStream;
use async_std::net::ToSocketAddrs;
use async_std::sync::Arc;
use futures::channel::mpsc;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

async fn client_writer(mut messages: Receiver<String>, stream: Arc<TcpStream>) -> Result<()> {
    let mut stream = &*stream;
    while let Some(msg) = messages.next().await {
        stream.write_all(msg.as_bytes()).await?;
    }
    Ok(())
}

use async_std::{net::TcpListener, prelude::*, task};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

//async fn server(addr: impl ToSocketAddrs + async_std::net::ToSocketAddrs) -> Result<()> {
async fn server(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        let _handle = task::spawn(client(stream));
    }
    Ok(())
}

async fn client(_stream: TcpStream) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    let fut = server("127.0.0.1:8080");
    task::block_on(fut)
}
