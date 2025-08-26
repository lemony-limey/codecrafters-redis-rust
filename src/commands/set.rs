use crate::{parser::resp_value::RESPValue, write_response, Database};
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
    write_response(writer, response).await
}
