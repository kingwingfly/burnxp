use anyhow::Result;
use clap::Parser;
use tagger::App;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// The root directory to scan for images
    #[arg(default_value = ".")]
    root: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // create app and run it
    let mut app = App::new(cli.root);
    app.run()?;
    Ok(())
}
