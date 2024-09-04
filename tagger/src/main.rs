use anyhow::Result;
use tagger::App;

fn main() -> Result<()> {
    // create app and run it
    let mut app = App::new();
    app.run()?;
    Ok(())
}
