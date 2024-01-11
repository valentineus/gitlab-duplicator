use clap::Parser;

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

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
