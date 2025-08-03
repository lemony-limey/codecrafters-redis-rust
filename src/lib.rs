// The following resource was infinitely helpful when developing the parser:
// https://dpbriggs.ca/blog/Implementing-A-Copyless-Redis-Protocol-in-Rust-With-Parsing-Combinators
// I have used much of this code.

use std::io::BufWriter;
use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
use crate::parser::{redis_command, RedisCommand};

pub mod parser;

/// Asynchronously handle an incoming connection.
pub async fn handle_connection(stream: &mut TcpStream) -> anyhow::Result<()>
{
    // Create a separate reader and writer
    let (read_stream, write_stream) = stream.split();
    let mut reader = BufReader::new(read_stream);
    let mut writer = tokio::io::BufWriter::new(write_stream);

    // Parse the input
    // TODO: Need entire input in a buffer or CRLF will split input into mini-lines and
    //      parsing will fail.
    let mut buffer = String::new();
    println!("hello");
    let _bytes_received = reader.read_to_string(&mut buffer).await?;
    println!("hello again");

    // Iterate over every line from the client.
    while !buffer.is_empty() && !buffer.eq("")
    {
        println!("{buffer}");
        let (remaining_input, command) = redis_command(buffer.clone())?;

        println!("{command:#?}");
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
                let response = format!("${}\r\n{}\r\n", message.chars().count(), message);
                let response = response.as_bytes();

                println!("response: {:#?}", response);

                // Write the contents of the buffer to the stream.
                writer.write_all(response)
                      .await
                      .with_context(|| "Could not write buffer contents to TCP stream")?;

                // Ensure the data is written to the stream immediately.
                writer.flush().await?;
            },
            _ => {}
        }

        buffer = remaining_input;
        let _bytes_received = reader.read_to_string(&mut buffer).await?;
        println!("buffer now: {}", buffer);
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