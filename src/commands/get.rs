use crate::{write_response, Database, NULL_BULK_STRING};
use anyhow::Context;
use chrono::Utc;
use nom::AsBytes;
use tokio::io::AsyncWriteExt;


pub async fn get(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
) -> anyhow::Result<()>
{
    // If the given key does not exist in the database, return a null bulk string.
    let database = database.lock()
                           .await;

    let buffer: String;
    let mut response: &[u8] = NULL_BULK_STRING;

    if let Some((value, expiry)) = database.get(&key)
    {
        // If value has not expired, return the null bulk string
        if let Some(expiry) = expiry && *expiry < Utc::now()
        {
            // response = NULL_BULK_STRING;
        }
        else  // Value has either not expired, or was not assigned an expiry value
        {
            buffer = value.to_resp_encoded_string()
                .expect("could not encode RESP value");
            response = buffer.as_bytes();
        }
    }

    write_response(writer, response.as_bytes()).await
}
