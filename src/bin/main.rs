use arrows_common::utils::type_of;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::stream::StreamExt;
use std::time::Duration;
#[async_std::main]
async fn main() -> std::io::Result<()> {
    //let store = &arrows::STORE;
    //println!("Store dir: {:?}", store.get_dir().await);
    let listener;
    match TcpListener::bind("0.0.0.0:7171").await {
        Ok(lsnr) => {
            listener = lsnr;
        }
        Err(err) => {
            eprintln!("Could not bind -> {:?}", err);
            return Err(err);
        }
    }
    println!("Listening on {}", listener.local_addr()?);
    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            handle_request(tcpstream).await;
        })
        .await;
    Ok(())
}

async fn transfer_actor(_stream: TcpStream) {}

async fn handle_request(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await;

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = std::fs::read_to_string(filename).unwrap();
    type_of(&contents);
    //let response = format!("{}{:?}", status_line, contents);
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    //stream.write(contents.as_bytes()).await;
    stream.write(response.as_bytes()).await;
    stream.flush().await;
}
