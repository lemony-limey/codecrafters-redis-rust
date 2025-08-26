use anyhow::Context;
use tokio::io::AsyncWriteExt;

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
          .with_context(|| "echo(): could not write buffer contents to TCP stream")?;

    // Ensure the data is written to the stream immediately.
    writer.flush().await?;
    Ok(())
}
