use anyhow::Context;
use tokio::io::AsyncWriteExt;
use crate::{Database, NULL_BULK_STRING};

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
