#![allow(unused_imports)]

use std::net::TcpStream;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use anyhow::{bail, Context};

fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming()
    {
        match stream
        {
            Ok(mut stream) => {
                println!("accepted new connection");
                let Ok(_) = process_stream(stream) else {
                    bail!("An error occurred when attempting to process the stream");
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    
    Ok(())
}

fn process_stream(mut stream: TcpStream) -> anyhow::Result<()>
{
    let stream_reader = BufReader::new(
        stream.try_clone()
            .with_context(|| "Could not clone a handle to the TCP stream")?
    );
    
    for line in stream_reader.lines()
    {
        let line = line.with_context(|| "Could not process line from the TCP stream")?;
        if line.eq_ignore_ascii_case("PING")
        {
            let buffer = b"+PONG\r\n";
            stream.write_all(buffer)
                  .with_context(|| "Could not write buffer contents to TCP stream")?;
        }
    }

    Ok(())
}
