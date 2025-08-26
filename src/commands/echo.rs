use anyhow::Context;
use tokio::io::AsyncWriteExt;
use crate::write_response;

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
    write_response(writer, response).await
}
