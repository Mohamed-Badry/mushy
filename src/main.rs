pub mod cli;
pub mod config;
pub mod daemon;
pub mod image_processing;
pub mod terminal;

use clap::Parser;
use std::fs;

use std::os::unix::process::CommandExt;
use std::process;

use cli::{Cli, Commands};

pub const RUN_FILE_PREFIX: &str = "mushy.";
pub const RUN_FILE_EXT: &str = ".run";

fn get_run_file_path() -> String {
    // SAFETY: `ttyname` returns a pointer to a static buffer or thread-local buffer
    // containing the null-terminated path of the terminal device open on the file descriptor.
    // We only pass `libc::STDOUT_FILENO`, which is valid.
    unsafe {
        let tty_ptr = libc::ttyname(libc::STDOUT_FILENO);
        if !tty_ptr.is_null() {
            let c_str = std::ffi::CStr::from_ptr(tty_ptr);
            let tty_name = c_str.to_string_lossy();
            let sanitized = tty_name.replace("/", "_");
            let mut path = std::env::temp_dir();
            path.push(format!("{}{}{}", RUN_FILE_PREFIX, sanitized, RUN_FILE_EXT));
            return path.to_string_lossy().into_owned();
        }
    }
    let mut path = std::env::temp_dir();
    path.push(format!("{}unknown{}", RUN_FILE_PREFIX, RUN_FILE_EXT));
    path.to_string_lossy().into_owned()
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Stop { all }) = cli.command {
        if all {
            if let Ok(entries) = fs::read_dir(std::env::temp_dir()) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(RUN_FILE_PREFIX) && file_name.ends_with(RUN_FILE_EXT) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
            println!("Stopped all mushy instances.");
        } else {
            let run_file = get_run_file_path();
            let _ = fs::remove_file(&run_file);
            println!("Stopped mushy in the current terminal.");
        }

        return;
    }

    let run_file = get_run_file_path();

    if cli.daemon {
        if let Err(e) = daemon::run(&cli) {
            eprintln!("Daemon error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Launch daemon
    if let Err(e) = fs::write(&run_file, "1") {
        eprintln!("Failed to write run file {}: {}", run_file, e);
        process::exit(1);
    }

    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get current executable path: {}", e);
            process::exit(1);
        }
    };

    let mut cmd = process::Command::new(exe);
    cmd.arg("--daemon");
    cmd.arg("--run-file");
    cmd.arg(&run_file);

    if let Some(cp) = &cli.config {
        cmd.arg("--config");
        cmd.arg(cp);
    }
    if let Some(gif) = &cli.gif {
        cmd.arg("--gif");
        cmd.arg(gif);
    }
    if let Some(size) = cli.size {
        cmd.arg("--size");
        cmd.arg(size.to_string());
    }
    if let Some(speed) = cli.speed {
        cmd.arg("--speed");
        cmd.arg(speed.to_string());
    }
    if cli.cw {
        cmd.arg("--cw");
    }

    // Detach from the current terminal so the user gets their prompt back.
    // SAFETY: `pre_exec` is unsafe because it runs in the context of the child process
    // between `fork()` and `exec()`. We must only call async-signal-safe functions here.
    // We uphold the safety contract by only calling `libc::setsid()`, which is a simple
    // POSIX system call that creates a new session and detaches the process from the TTY.
    // We avoid all memory allocations, locks, or complex Rust standard library calls.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    if let Err(e) = cmd.spawn() {
        eprintln!("Failed to start daemon: {}", e);
        process::exit(1);
    }
}
