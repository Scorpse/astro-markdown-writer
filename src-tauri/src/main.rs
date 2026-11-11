#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod github;

use github::{Connection, Post, Publication, SiteSetup};
use pulldown_cmark::{html, Event, Options, Parser};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct Session(Mutex<Option<Connection>>);

#[derive(Serialize)]
struct Connected {
    repo: String,
    branch: String,
}

async fn background<T: Send + 'static>(
    job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|_| "The background operation could not finish.".to_owned())?
}

impl Session {
    fn current(&self) -> Result<Connection, String> {
        self.0
            .lock()
            .map_err(|_| "Restart Writer to reconnect.")?
            .clone()
            .ok_or_else(|| "Connect a repository first.".into())
    }
}

#[tauri::command]
async fn connect(
    session: State<'_, Session>,
    url: String,
    token: String,
) -> Result<Connected, String> {
    let connection = background(move || Connection::connect(&url, token)).await?;
    let result = Connected {
        repo: connection.repo.clone(),
        branch: connection.branch.clone(),
    };
    *session
        .0
        .lock()
        .map_err(|_| "Restart Writer to reconnect.")? = Some(connection);
    Ok(result)
}

#[tauri::command]
async fn setup_pages(session: State<'_, Session>) -> Result<SiteSetup, String> {
    let connection = session.current()?;
    background(move || connection.setup_pages()).await
}

#[tauri::command]
async fn publish(session: State<'_, Session>, post: Post) -> Result<Publication, String> {
    let connection = session.current()?;
    background(move || connection.publish(post)).await
}

#[tauri::command]
fn preview(source: &str) -> String {
    let parser = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        event => event,
    });
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

fn main() {
    tauri::Builder::default()
        .manage(Session::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            connect,
            setup_pages,
            publish,
            preview
        ])
        .run(tauri::generate_context!())
        .expect("Astro Markdown Writer could not start");
}
