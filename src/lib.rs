

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::parser::{redis_commands, RedisCommand};
use anyhow::{anyhow, Context};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub mod parser;

type Database = Arc<Mutex<HashMap<String, Bytes>>>;

/// Asynchronously handle an incoming connection.
///
/// Commands are parsed from the string received from the client, and added to a queue
/// where they will then be executed in the order that they were received.
pub async fn handle_connection(
    stream: &mut TcpStream,
    database: Database,
) -> anyhow::Result<()>
{
    // Create a separate reader and writer
    let (read_stream, write_stream) = stream.split();
    let mut reader = BufReader::new(read_stream);
    let mut writer = tokio::io::BufWriter::new(write_stream);

    // Parse the input
    let mut buffer = BytesMut::with_capacity(1024);
    let _bytes_received = reader.read_buf(&mut buffer).await?;

    // Iterate over every line from the client.
    while !buffer.is_empty() && !buffer.eq("")
    {
        println!("{}", str::from_utf8(&buffer)?);

        // Parse as many commands from the input as possible.
        let (remaining_input, commands) = redis_commands(buffer.clone().into())?;
        // If no commands could be parsed, return an error.
        let Some(commands) = commands else {
            return Err(anyhow!("Incomplete input, cannot parse commands"));
        };
        println!("{:#?}", commands);

        // Execute every command in the command queue
        for command in commands
        {
            println!("{command:#?}");
            match command
            {
                RedisCommand::Ping => ping(&mut writer).await?,
                RedisCommand::Echo { message } => echo(&mut writer, message).await?,
                _ => {}
            }
        }

        buffer = BytesMut::from(remaining_input.as_bytes());
        let _bytes_received = reader.read_buf(&mut buffer).await?;
        println!("buffer now: {:#?}", buffer);
    }

    // Close the output stream
    writer.shutdown().await?;

    Ok(())
}

/// Handler for the PING command.
///
/// Sends the byte string "+PONG\r\n" in response whenever a PING is received.
///
///
pub async fn ping(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
) -> anyhow::Result<()>
{
    let buffer = b"+PONG\r\n";

    // Write the contents of the buffer to the stream.
    writer.write_all(buffer)
          .await
          .with_context(|| "Could not write buffer contents to TCP stream")?;

    // Ensure the data is written to the stream immediately.
    writer.flush().await?;

    Ok(())
}

/// Handler for the ECHO command.
pub async fn echo(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    message: String,
) -> anyhow::Result<()>
{
    let response = format!("${}\r\n{}\r\n", message.chars().count(), message);
    let response = response.as_bytes();

    println!("response: {:#?}", response);

    // Write the contents of the buffer to the stream.
    writer.write_all(response)
          .await
          .with_context(|| "Could not write buffer contents to TCP stream")?;

    // Ensure the data is written to the stream immediately.
    writer.flush().await?;

    Ok(())
}

pub async fn set(

) -> anyhow::Result<()>
{
    todo!()
}

pub async fn get(

) -> anyhow::Result<()>
{
    todo!()
}
