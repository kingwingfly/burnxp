use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tagger::App;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// The output file path
    #[clap(short, long, default_value = "tags.json")]
    output: PathBuf,
    /// The file to cache the comparison results (relative paths are cached)
    #[clap(short, long, default_value = "cache.bin")]
    cache: PathBuf,
    /// The root directory to scan for images
    root: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // create app and run it
    let mut app = App::new(cli.root, cli.output, cli.cache);
    app.run()?;
    Ok(())
}
