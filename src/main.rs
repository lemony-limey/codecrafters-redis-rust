#![allow(unused_imports)]

use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    io::{BufReader, BufWriter},
};
use anyhow::{bail, Context};

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // The event loop for the server
    loop
    {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(stream).await.unwrap();
        });
    }
}

/// Asynchronously handle an incoming connection.
async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()>
{
    // Create a separate reader and writer
    let (read_stream, write_stream) = stream.split();
    let reader = BufReader::new(read_stream);
    let mut writer = BufWriter::new(write_stream);

    // Split the input into individual lines.
    let mut lines = reader.lines();

    // Iterate over every line from the client.
    while let Ok(Some(line)) = lines.next_line().await
    {
        if line.eq_ignore_ascii_case("PING")
        {
            let buffer = b"+PONG\r\n";
            
            // Write the contents of the buffer to the stream.
            writer.write_all(buffer)
                .await
                .with_context(|| "Could not write buffer contents to TCP stream")?;
            
            // Ensure the data is written to the stream immediately.
            writer.flush().await?;
        }
    }
    
    // Close the output stream
    writer.shutdown().await?;

    Ok(())
}
