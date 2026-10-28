use std::env;
use mini_redis::{Connection, Frame, Command::{self, Get, Set}};
use reqwest;
use std::io::Write;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::{fs, io::BufReader};
use whereis;
use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Bitte Suchbegriff eingeben");
    } else {
        let query = &args[1];
        whereis::search(query);
    } 

    let app = Router::new()
        .route("/",get(|| async {
            Html(include_str!("../index.html"))
        }))
    .route("/search",get(search));

    let listener = TcpListener::bind("127.0.0.1:7878")
        .await
        .unwrap();

    println!("Listening on http://127.0.0.1:7878");

    axum::serve(listener, app).await.unwrap();
}

async fn search(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("query").map(String::as_str).unwrap_or("");

    let results = whereis::search(query);

    let html = format!(
        r#"<DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Results</title>
        </head>
        <body>
            <h1>Results for {}</h1>
            <pre>{}</pre>
            <a href="/">Back to search</a>
        </body>
        </html>"#,
        escape_html(query),
        escape_html(&results),
    );

    Html(html)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace("'", "&quot;")
        .replace('\"', "&#39;")
}

async fn process(stream: TcpStream) {
    let mut connection = Connection::new(stream);
    let (status_line, filename) = ("HTTP/1.1 200 OK", "index.html");

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContents-Length: {length}\r\n\r\n{contents}");

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                println!("Got a set");
                Frame::Simple(response.clone())
            }
            Get(cmd) => {
                println!("Got a get");
                Frame::Simple(response.clone())
            }
            _ => {
                Frame::Simple(response.clone())
            }
        };
        connection.write_frame(&response).await.unwrap();
    }
}
