use std::env;
use whereis;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use mini_redis::{client, Result};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Bitte Suchbegriff eingeben");
    } else {
        let query = &args[1];
        whereis::search(query);
    }

    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}
