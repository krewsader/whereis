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
        .route("/add", get(|| async { Html(include_str!("../add.html")) }))
        .route("/search", get(search))
        .route("/added", post(add))
        .route("/modify", get(modify_page))
        .route("/modified", post(modify))
        .route("/delete", get(delete_page))
        .route("/deleted", post(delete));

    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    println!("Listening on http://127.0.0.1:7878");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn search(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let query = params.get("query").map(String::as_str).unwrap_or("");

    let results = format!(r#"{}"#, whereis::search(query));

    let html = format!(
        r#"<DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Results</title>

            <style>
                body {{
                    margin: 0;
                    font-family: Arial, sans-serif;
                }}

                h2 {{
                    margin: 4px;
                }}

                pre {{
                    margin: 0px;
                    padding: 0px;
                }}

               .container {{
                   display: flex;
                   flex-direction: column;
                   align-items: center;
                   justify-content: center;
                   min-height: 100vh;
               }} 

               .results {{
                    font-family: Arial, sans-serif;
                    display: flex;
                    flex-direction: column;
                    gap: 4px;
                    margin-top: 4px;
                    margin-bottom: 16px;
               }}

               .result {{
                font-family: Arial, sans-serif;
               }}
            </style>
        </head>
        <body>
        <div class="container">
            <h1>Results for "{}"</h1>
            <pre>{}</pre>
            <a href="/">Back to search</a>
        </div>
        </body>
        </html>"#,
        escape_html(query),
        results,
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
            content = format!(
                "Sucessfully added department {} at {} with example room {}",
                name, location, room
            );
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
            <style>
                body {{
                    margin: 0;
                    font-family: Arial, sans-serif;
                }}

               .container {{
                   display: flex;
                   flex-direction: column;
                   align-items: center;
                   justify-content: center;
                   min-height: 100vh;
               }} 
            </style>

        </head>
        <body>
            <div class="container">
            <h1>{}</h1>
            <a href="/">Back to search</a>
            </div>
        </body>
        </html>"#,
        title, content,
    );

    Html(html)
}

async fn modify_page(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let name = params.get("name").map(String::as_str).unwrap_or("");
    let name = escape_html(name);

    let html = format!(include_str!("../modify.html"), name, name);

    Html(html)
}

async fn modify(Form(params): Form<HashMap<String, String>>) -> Html<String> {
    let name = params.get("name").map(String::as_str).unwrap_or("");
    let location = params.get("location").map(String::as_str).unwrap_or("");
    let room = params.get("room").map(String::as_str).unwrap_or("None");

    let title;
    let content;

    match whereis::modify(name.to_string(), location.to_string(), room.to_string()) {
        Ok(_) => {
            title = format!("Success");
            content = format!(
                "Sucessfully updated location of {} to {} with example room {}",
                name, location, room
            );
        }
        Err(e) => {
            title = format!("Failure");
            content = format!("Error updating department: {}", e);
        }
    };

    result_page(&title, &content)
}

async fn delete_page(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let name = params.get("name").map(String::as_str).unwrap_or("");
    let name = escape_html(name);

    let html = format!(include_str!("../delete.html"), name, name);

    Html(html)
}

async fn delete(Form(params): Form<HashMap<String, String>>) -> Html<String> {
    let name = params.get("name").map(String::as_str).unwrap_or("");
    let name = escape_html(name);

    let title;
    let content;

    match whereis::delete(name.to_string()) {
        Ok(_) => {
            title = format!("Success");
            content = format!("Sucessfully deleted department {}", name);
        }
        Err(e) => {
            title = format!("Failure");
            content = format!("Error updating department: {}", e);
        }
    };

    result_page(&title, &content)
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace("'", "&quot;")
        .replace('\"', "&#39;")
}
