use anyhow::Context;
use tokio::io::AsyncWriteExt;

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
          .with_context(|| "ping(): could not write buffer contents to TCP stream")?;

    // Ensure the data is written to the stream immediately.
    writer.flush().await?;
    Ok(())
}
