#![allow(unused_imports)]

use std::io::Write;
use std::net::TcpListener;
use anyhow::Context;

fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming()
    {
        match stream
        {
            Ok(mut stream) => {
                println!("accepted new connection");
                let buffer = b"+PONG\r\n";
                stream.write_all(buffer)
                    .with_context(|| "Could not write buffer contents to TCP stream")?
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    
    Ok(())
}
