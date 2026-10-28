use axum::{Router, extract::Query, response::Html, routing::get};
use std::collections::HashMap;
use tokio::net::TcpListener;
use whereis;

#[tokio::main]
async fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("Bitte Suchbegriff eingeben");
    // } else {
    //     let query = &args[1];
    //     whereis::search(query);
    // }

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/search", get(search));

    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    println!("Listening on http://127.0.0.1:7878");

    axum::serve(listener, app).await.unwrap();
}

async fn search(Query(params): Query<HashMap<String, String>>) -> Html<String> {
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
