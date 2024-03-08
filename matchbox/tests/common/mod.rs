use std::{
    process::Command,
    time::{Duration, Instant},
};

pub fn ping(ip_address: impl AsRef<str>) -> anyhow::Result<()> {
    let mut cmd = Command::new("timeout");
    let output = cmd
        .args(["1", "ping", "-c", "1", ip_address.as_ref()])
        .output()?;

    match output.status.success() {
        true => Ok(()),
        false => anyhow::bail!(
            "failed to ping the {}. {}",
            ip_address.as_ref(),
            String::from_utf8_lossy(&output.stdout)
        ),
    }
}

pub fn wait_until(duration: Duration, func: impl Fn() -> anyhow::Result<()>) -> anyhow::Result<()> {
    let start = Instant::now();
    while Instant::now() < start + duration {
        match func() {
            Ok(_) => return Ok(()),
            Err(e) => println!("got error from function: {e:?}"),
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    anyhow::bail!("function did not return true after {duration:?}");
}
