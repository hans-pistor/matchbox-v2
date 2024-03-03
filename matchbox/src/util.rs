use std::{path::Path, process::Command};

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) {
    let mut cmd = Command::new("cp");
    cmd.args([from.as_ref(), to.as_ref()]);

    cmd.output().unwrap();
}
