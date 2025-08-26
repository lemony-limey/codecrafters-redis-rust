use crate::{
    parser::resp_value::RESPValue,
    Database,
};
use anyhow::Context;
use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt;

pub async fn set(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
    value: RESPValue,
    px: Option<DateTime<Utc>>,
) -> anyhow::Result<()>
{
    let mut database = database.lock()
        .await;

    let _ = database.insert(key, (value, px));

    // Indicate success
    let response = b"+OK\r\n";
    writer.write_all(response)
          .await
          .with_context(|| "set(): could not write buffer contents to TCP stream")?;

    // Ensure that the desired output is written to the stream immediately.
    writer.flush().await?;
    Ok(())
}
