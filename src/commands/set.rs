use std::collections::VecDeque;
use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use tokio::io::AsyncWriteExt;
use crate::{
    Database,
    parser::resp_value::RESPValue,
};
use crate::commands::redis_command::RedisCommand;
use crate::parser::resp_error::RESPError;

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


