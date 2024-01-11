#[macro_use]
extern crate dotenv_codegen;
extern crate git2;

use std::error::Error;
use std::path::Path;

use clap::Parser;
use git2::{Cred, RemoteCallbacks};

/// Create a mirror of a repository
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to save the repository to
    #[arg(short, long)]
    path: String,

    /// The URL of the repository to clone
    #[arg(short, long)]
    repo: String,
}

fn get_project_url(repo: &str) -> String {
    format!("https://{}/{}.git", dotenv!("GITLAB_URL"), repo)
}

fn get_remote_url(repo: &str) -> String {
    format!(
        "https://{}:{}@{}/{}",
        dotenv!("REMOTE_USERNAME"),
        dotenv!("REMOTE_PASSWORD"),
        dotenv!("REMOTE_URL"),
        repo
    )
}

/* Clone project from Gitlab */
fn clone_remote_project(repo: &str, path: &str) {
    // Callbacks for authentication
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(dotenv!("GITLAB_USERNAME"), dotenv!("GITLAB_PASSWORD"))
    });

    // Prepare fetch options
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(cb);

    // Prepare builder
    let mut builder = git2::build::RepoBuilder::new();
    builder.remote_create(|repo, name, url| repo.remote_with_fetch(name, url, "+refs/*:refs/*"));

    builder.fetch_options(fetch_options);
    builder.bare(true);

    // Clone
    builder.clone(repo, Path::new(&path)).unwrap();
}

/* Get project ID from Gitlab */
fn get_project_id(project_name: &str) -> Result<u64, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let encoded_name = urlencoding::encode(project_name);

    let api_url = format!(
        "https://{}/api/v4/projects/{}",
        dotenv!("GITLAB_URL"),
        encoded_name
    );

    let response = client
        .get(&api_url)
        .bearer_auth(dotenv!("GITLAB_TOKEN"))
        .send()?;

    if response.status().is_success() {
        let project_info: serde_json::Value = response.json()?;
        Ok(project_info["id"].as_u64().unwrap())
    } else {
        Err("Failed to get project ID".into())
    }
}

/* Add mirror to Gitlab */
fn add_gitlab_mirror(repo_id: u64, mirror_url: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();

    let api_url = format!(
        "https://{}/api/v4/projects/{}/remote_mirrors",
        dotenv!("GITLAB_URL"),
        repo_id
    );

    let response = client
        .post(&api_url)
        .bearer_auth(dotenv!("GITLAB_TOKEN"))
        .json(&serde_json::json!({
            "url": mirror_url,
            "enabled": true,
            "keep_divergent_refs": false,
            "only_protected_branches": false,
        }))
        .send()?;

    dbg!(response.status());
    if response.status().is_success() {
        Ok(())
    } else {
        Err("Failed to add mirror".into())
    }
}

fn main() {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let project_id = get_project_id(&args.repo).unwrap();
    let mirror_url = get_remote_url(&args.path);
    add_gitlab_mirror(project_id, &mirror_url).unwrap();

    dbg!(&project_id);
    dbg!(&mirror_url);

    clone_remote_project(&get_project_url(&args.repo), &args.path);
}
