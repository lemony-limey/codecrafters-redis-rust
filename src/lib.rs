use std::io::BufWriter;
use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
use crate::parser::{redis_command, RedisCommand};

pub mod parser;

/// Asynchronously handle an incoming connection.
pub async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()>
{
    // Create a separate reader and writer
    let (read_stream, write_stream) = stream.split();
    let reader = BufReader::new(read_stream);
    let mut writer = tokio::io::BufWriter::new(write_stream);

    // Parse the input
    let mut lines = reader.lines();

    // Iterate over every line from the client.
    while let Ok(Some(line)) = lines.next_line().await
    {
        let Ok((_, command)) = redis_command(line)?;

        match command
        {
            RedisCommand::Ping => {
                let buffer = b"+PONG\r\n";

                // Write the contents of the buffer to the stream.
                writer.write_all(buffer)
                      .await
                      .with_context(|| "Could not write buffer contents to TCP stream")?;

                // Ensure the data is written to the stream immediately.
                writer.flush().await?;
            },
            RedisCommand::Echo { message } => {
                let buffer = format!("${}\r\n{}\r\n", message.len(), message);
                let buffer = buffer.as_bytes();

                // Write the contents of the buffer to the stream.
                writer.write_all(buffer)
                      .await
                      .with_context(|| "Could not write buffer contents to TCP stream")?;

                // Ensure the data is written to the stream immediately.
                writer.flush().await?;
            },
            _ => {}
        }
    }

    // Close the output stream
    writer.shutdown().await?;

    Ok(())
}

// pub fn ping(writer: BufWriter<WriteHalf<TcpStream>>)
// {
//     let buffer = b"+PONG\r\n";
//
//     // Write the contents of the buffer to the stream.
//     writer.write_all(buffer)
//           .await
//           .with_context(|| "Could not write buffer contents to TCP stream")?;
//
//     // Ensure the data is written to the stream immediately.
//     writer.flush().await?;
// }