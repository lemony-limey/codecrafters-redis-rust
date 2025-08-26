use std::collections::HashMap;
use std::fmt::format;
use std::io::Write;
use std::sync::{Arc, MutexGuard};
use crate::parser::{redis_commands, RESPValue, RedisCommand};
use anyhow::{anyhow, Context};
use bytes::{BufMut, Bytes, BytesMut};
use nom::AsBytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub mod parser;

const NULL_BULK_STRING: &[u8] = b"$-1\r\n";

type Database = Arc<Mutex<HashMap<String, RESPValue>>>;


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
        println!("{commands:#?}");
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
                RedisCommand::Set { key, value } => {
                    set(&mut writer, database.clone(), key, value).await?
                },
                RedisCommand::Get { key } => get(&mut writer, database.clone(), key).await?,
            }
        }

        buffer = BytesMut::from(remaining_input.as_bytes());
        let _bytes_received = reader.read_buf(&mut buffer).await?;
        println!("buffer now: {:#?}", buffer);

        {
            println!("database now: {:#?}", database.lock().await);
        }
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
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
    value: RESPValue,
) -> anyhow::Result<()>
{
    let mut database = database.lock()
        .await;

    let _ = database.insert(key, value);

    // Indicate success
    let response = b"+OK\r\n";
    writer.write_all(response)
        .await
        .with_context(|| "Could not write buffer contents to TCP stream")?;

    // Ensure that the desired output is written to the stream immediately.
    writer.flush().await?;
    Ok(())
}


pub async fn get(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
) -> anyhow::Result<()>
{
    // If the given key does not exist in the database, return a null bulk string.
    let database = database.lock()
        .await;

    let mut buffer: String = String::new();
    let response: &[u8];

    if let Some(value) = database.get(&key)
    {
        buffer = value.to_resp_encoded_string()
            .expect("could not encode RESP value");
        response = buffer.as_bytes();
    }
    else
    {
        response = NULL_BULK_STRING;
    }

    writer.write_all(response)
        .await
        .with_context(|| "")?;

    writer.flush().await?;
    Ok(())
}
