use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use axum::{
    Router,
    extract::{Form, Query},
    response::Html,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
};
use std::{io::Write, net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use whereis;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(10)
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/add", get(|| async { Html(include_str!("../add.html")) }))
        .route("/search", get(search))
        .route("/added", post(add))
        .route("/modify", get(modify_page))
        .route("/modified", post(modify))
        .route("/delete", get(delete_page))
        .route("/deleted", post(delete))
        .layer(GovernorLayer::new(governor_conf));
    let exe_path = std::env::current_exe().expect("Failed to get executable path");

    let exe_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");

    let env_path = exe_dir.join(".env");

    dotenvy::from_path(env_path).expect("Failed to load .env file");
    println!("Working directory: {:?}", std::env::current_dir());
    println!("PORT env: {:?}", std::env::var("PORT"));

    let cert_path = std::env::var("TLS_CERT_PATH").expect("TLS_CERT_PATH not set");
    let key_path = std::env::var("TLS_KEY_PATH").expect("TLS_KEY_PATH not set");
    println!("TLS_CERT_PATH: {:?}", cert_path);
    println!("TLS_KEY: {:?}", key_path);
    let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(|e| {println!("Error configuring TLS: {:#}", e); e})?;


    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8444".to_string())
        .parse()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    /*     let listener = TcpListener::bind(addr).await?; */

    println!("Listening on https://0.0.0.0:{}", port);

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    /*     axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?; */

    Ok(())
}

async fn search(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let query = params.get("query").map(String::as_str).unwrap_or("");

    if query.len() > 256 {
        return Html(format!(
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
            <h1>Query too long! Try a different one</h1>
            <a href="/">Back to search</a>
        </div>
        </body>
        </html>"#
        ));
    }

    let query = &query.to_lowercase() as &str;

    let easter_eggs = HashSet::from([
        "god",
        "gott",
        "jeffrey",
        "epstein",
        "jeffrey epstein",
        "waldo",
        "walter",
        "rick astley",
        "my pants",
        "meine hosen",
        "meine hose",
        "pascal",
        "big p",
        "kenobi",
        "obi-wan kenobi",
        "obi wan kenobi",
    ]);

    if easter_eggs.contains(query) {
        return easter_egg(query);
    }

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
    let name = escape_html(name);
    let location = params.get("location").map(String::as_str).unwrap_or("");
    let location = escape_html(location);
    let room = params.get("room").map(String::as_str).unwrap_or("None");
    let room = escape_html(room);
    let password = params.get("password").map(String::as_str).unwrap_or("");

    let title;
    let content;

    if !check_password(password) {
        return result_page("Failure", "Wrong password");
    }

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
    let password = params.get("password").map(String::as_str).unwrap_or("");

    let title;
    let content;

    if !check_password(password) {
        return result_page("Failure", "Wrong password");
    }

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
    let password = params.get("password").map(String::as_str).unwrap_or("");

    if !check_password(password) {
        return result_page("Failure", "Wrong password");
    }

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

fn check_password(input: &str) -> bool {
    let hash = fs::read_to_string("hash.txt").expect("Error reading password file");
    let hash = hash.trim();
    let parsed_hash = PasswordHash::new(&hash).expect("Error creating hash object");

    Argon2::default()
        .verify_password(input.as_bytes(), &parsed_hash)
        .is_ok()
}

fn set_password(input: String) -> Result<(), Box<dyn std::error::Error>> {
    let password = input.as_bytes();

    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password)?.to_string();
    let parsed_hash = PasswordHash::new(&hash)?;
    assert!(
        Argon2::default()
            .verify_password(password, &parsed_hash)
            .is_ok()
    );

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("hash.txt")
        .expect("unable to open hash.txt");

    writeln!(file, "{}", hash)?;
    Ok(())
}

fn easter_egg(query: &str) -> Html<String> {
    let url = match query {
        "god" | "gott" => "https://www.bibleserver.com/",
        "jeffrey" | "epstein" | "jeffrey epstein" => "https://maps.app.goo.gl/yHwPnVNUE7i2th9u8",
        "waldo" | "walter" => "https://whereiswaldo.com/level1.html",
        "my pants" | "meine hosen" | "meine hose" => "https://www.youtube.com/watch?v=ZhaT7F7lMEY",
        "kenobi" | "obi wan kenobi" | "obi-wan kenobi" => {
            "https://media.tenor.com/WuOwfnsLcfYAAAAC/star-wars-obi-wan-kenobi.gif"
        }
        "pascal" | "big p" => "https://www.admin.ch/de",
        _ => "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script>
                    window.top.location.replace("{}");
                </script>
            </head>

            <body>
                <p>Finding your deepest desires...</p>
            </body>
        </html>
    "#,
        url
    ))
}
