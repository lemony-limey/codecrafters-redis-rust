use std::cmp::max;
use anyhow::{anyhow, Context};
use tokio::io::AsyncWriteExt;
use crate::{write_response, Database};
use crate::parser::resp_value::RESPValue;

const EMPTY_ARRAY: &[u8] = b"*0\r\n";


pub async fn lrange(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
    database: Database,
    key: String,
    mut start_inclusive: i64,
    mut end_inclusive: i64,
) -> anyhow::Result<()>
{
    let database = database.lock().await;

    let mut response: String;

    // Get the linked list from the database
    let Some((RESPValue::List(linked_list), _expiry)) = database.get(&key) else {
        return write_response(writer, EMPTY_ARRAY).await;
    };

    let list_len = linked_list.len();
    while start_inclusive < 0
    {
        start_inclusive += list_len as i64;
    }
    while end_inclusive < 0
    {
        end_inclusive += list_len as i64;
    }
    while end_inclusive > list_len as i64
    {
        end_inclusive = list_len as i64 - 1
    }
    if start_inclusive >= list_len as i64 || start_inclusive > end_inclusive
    {
        return write_response(writer, EMPTY_ARRAY).await;
    }

    response = format!("*{}\r\n", end_inclusive - start_inclusive + 1);

    let response = linked_list.iter()
        .skip(start_inclusive as usize)
        .take((end_inclusive - start_inclusive + 1) as usize)
        .fold(&mut response, |acc, next_value| {
            acc.push_str(
                next_value.to_resp_encoded_string()
                    .expect("could not encode resp value")
                    .as_str()
            );
            acc
        });

    write_response(writer, response.as_bytes()).await
}