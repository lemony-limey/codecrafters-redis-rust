use crate::write_response;


/// Handler for the PING command.
///
/// Sends the byte string "+PONG\r\n" in response whenever a PING is received.
///
///
pub async fn ping(
    writer: &mut tokio::io::BufWriter<tokio::net::tcp::WriteHalf<'_>>,
) -> anyhow::Result<()>
{
    let response = b"+PONG\r\n";

    write_response(writer, response).await
}
