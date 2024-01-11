#[macro_use]
extern crate dotenv_codegen;
extern crate git2;

use std::path::Path;

use clap::Parser;
use git2::{Cred, RemoteCallbacks};
use gitlab::api::{projects, Query};
use gitlab::Gitlab;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    http_url_to_repo: String,
}

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

    /// The username to use for authentication
    #[arg(long)]
    username: String,

    /// The password to use for authentication
    #[arg(long)]
    password: String,
}

/* Get project info from Gitlab */
fn get_project(url: &str) -> Project {
    let client = Gitlab::new(dotenv!("GITLAB_URL"), dotenv!("GITLAB_TOKEN")).unwrap();

    let endpoint = projects::Project::builder().project(url).build().unwrap();
    let project: Project = endpoint.query(&client).unwrap();

    project
}

/* Clone project from Gitlab */
fn clone_project(repo: &str, path: &str, username: &str, password: &str) {
    // Callbacks for authentication
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username, password)
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

fn main() {
    dotenv::dotenv().ok();

    let args = Args::parse();
    let project = get_project(&args.repo);

    clone_project(
        &project.http_url_to_repo,
        &args.path,
        &args.username,
        &args.password,
    );
}
