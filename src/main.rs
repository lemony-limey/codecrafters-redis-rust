#![allow(unused_imports)]

use anyhow::{bail, Context};
use codecrafters_redis::handle_connection;
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    io::{BufReader, BufWriter},
};

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // The event loop for the server
    loop
    {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let result = handle_connection(stream).await.unwrap();
            println!("{result:#?}");
        });
    }
}
