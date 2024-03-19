use std::os::unix::fs;

use anyhow::{Context, Result};
use libc;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    if std::fs::metadata("./tmp-cc").is_err() {
        std::fs::create_dir("./tmp-cc")?;
        std::fs::create_dir("./tmp-cc/dev")?;
        std::fs::write("./tmp-cc/dev/null", b"")?;
        std::fs::create_dir_all("./tmp-cc/usr/local/bin")?;
        std::fs::copy(command, format!("./tmp-cc/usr/local/bin/docker-explorer"))
            .expect("Failed to copy");
    }

    fs::chroot("./tmp-cc")?;
    std::env::set_current_dir("/")?;

    unsafe {
        #[cfg(target_os = "linux")]
        libc::unshare(libc::CLONE_NEWPID);
    }

    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let std_out = std::str::from_utf8(&output.stdout)?;
    print!("{}", std_out);

    let std_err = std::str::from_utf8(&output.stderr)?;
    eprint!("{}", std_err);

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(1);
        std::process::exit(exit_code);
    }

    Ok(())
}
