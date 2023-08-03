use std::path::Path;

use anyhow::{bail, Result};
use tokio::process::Command;

pub async fn pdftoppm(path: &Path, dest: &Path) -> Result<()> {
    let output = Command::new("pdftoppm")
        .args(["-png", "-singlefile"])
        .arg(path)
        .arg(dest)
        .kill_on_drop(true)
        .output()
        .await?;

    if !output.status.success() {
        bail!(
            "failed to get pdf: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
