mod explorer;
mod node;

use anyhow::Result;
use explorer::Explorer;

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;
    Explorer::new(cwd)?.run()
}