use axum::{Router, extract::Form, extract::Query, response::Html, routing::get, routing::post};
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
        .route("/search", get(search))
        .route("/add", post(add));

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

async fn add(Form(params): Form<HashMap<String, String>>) -> Html<String> {
    let name = params.get("name").map(String::as_str).unwrap();
    let location = params.get("location").map(String::as_str).unwrap_or("");
    let room = params.get("room").map(String::as_str).unwrap_or("None");

    match whereis::add(name.to_string(), location.to_string(), room.to_string()) {
        Ok(_) => {}
        Err(e) => {
            let html = format!(
                r#"<DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8"
            <title>Failure</title>
        </head>
        <body>
            <h1>Error: {}</h1>
            <a href="/">Back to search</a>
        </body>
        </html>"#,
                e
            );

            return Html(html);
        }
    };

    let html = format!(
        r#"<DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8"
            <title>Success</title>
        </head>
        <body>
            <h1>Sucessfully added department {} at {} with example room {}</h1>
            <a href="/">Back to search</a>
        </body>
        </html>"#,
        name, location, room,
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
