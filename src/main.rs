#[macro_use]
extern crate dotenv_codegen;

use clap::Parser;
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
    url: String,
}

/* Get project info from Gitlab */
fn get_project(url: &str) -> Project {
    let gl_client = Gitlab::new(dotenv!("GITLAB_URL"), dotenv!("GITLAB_TOKEN")).unwrap();

    let gl_endpoint = projects::Project::builder().project(url).build().unwrap();
    let gl_project: Project = gl_endpoint.query(&gl_client).unwrap();

    gl_project
}

fn main() {
    dotenv::dotenv().ok();

    let args = Args::parse();
    println!("{:?}", args);

    let project = get_project(&args.url);
    dbg!(project);
}
