use std::{path::Path, process::Command};

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut cmd = Command::new("cp");
    cmd.args([from.as_ref(), to.as_ref()]);

    let output = cmd.output().unwrap();
    if output.status.success() {
        return Ok(());
    }

    anyhow::bail!(
        "failed to copy file from {} to {}. {}",
        from.as_ref().display(),
        to.as_ref().display(),
        String::from_utf8_lossy(&output.stdout)
    )
}
