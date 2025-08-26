use codecrafters_redis::handle_connection;
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::Mutex,
};


/// The entry point for the database program.
#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let database = Arc::new(Mutex::new(HashMap::new()));

    // The event loop for the server
    loop
    {
        let (mut stream, _addr) = listener.accept().await?;

        // Clone a handle to the database that can be passed safely to the new thread
        let database_handle = Arc::clone(&database);

        tokio::spawn(async move {
            let result = handle_connection(
                &mut stream,
                database_handle,
            ).await;
            stream.shutdown().await.unwrap();
            println!("result: {result:#?}");
        });
    }
}
