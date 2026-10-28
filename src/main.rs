use axum::{Router, extract::Form, extract::Query, response::Html, routing::get, routing::post};
use std::collections::HashMap;
use tokio::net::TcpListener;
use whereis;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    println!("Listening on http://127.0.0.1:7878");

    axum::serve(listener, app).await?;

    Ok(())
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
    let name = params.get("name").map(String::as_str).unwrap_or("");
    let location = params.get("location").map(String::as_str).unwrap_or("");
    let room = params.get("room").map(String::as_str).unwrap_or("None");

    let title;
    let content;

    match whereis::add(name.to_string(), location.to_string(), room.to_string()) {
        Ok(_) => {
            title = format!("Success");
            content = format!("Sucessfully added department {} at {} with example room {}", name, location, room);
        }
        Err(e) => {
            title = format!("Failure");
            content = format!("Error adding department: {}", e);
        }
    };

    result_page(&title, &content)
}

fn result_page(title: &str, content: &str) -> Html<String> {
    let html = format!(
        r#"<DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8"
            <title>{}</title>
        </head>
        <body>
            <h1>{}</h1>
            <a href="/">Back to search</a>
        </body>
        </html>"#,
        title, content,
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
