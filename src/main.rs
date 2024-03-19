use std::{os::unix::fs, path::Path};

use anyhow::{Context, Result};

use crate::docker_hub::api::download_image;

mod docker_hub;

const CHROOT: &str = "./temp";

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];

    if std::fs::metadata(CHROOT).is_err() {
        std::fs::create_dir(CHROOT)?;
        std::fs::create_dir(format!("{CHROOT}/dev"))?;
        std::fs::File::create(format!("{CHROOT}/dev/null"))?;
        std::fs::create_dir_all(format!("{CHROOT}/usr/local/bin"))?;
        std::fs::copy(command, format!("{CHROOT}/usr/local/bin/docker-explorer"))
            .expect("Failed to copy");
    }

    let path = Path::new(CHROOT).to_path_buf();
    download_image(&path, image).await?;

    fs::chroot(CHROOT)?;
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
