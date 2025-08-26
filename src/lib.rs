pub mod parser;
mod commands;

use anyhow::anyhow;
use bytes::BytesMut;
use crate::{
    commands::{
        echo::echo,
        get::get,
        ping::ping,
        set::set,
    },
    parser::{
        redis_commands,
        resp_value::RESPValue,
        redis_command::RedisCommand,
    },
};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
};

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
