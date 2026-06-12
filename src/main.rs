pub mod cli;
pub mod config;
pub mod daemon;
pub mod image_processing;
pub mod terminal;

use clap::Parser;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::process;

use cli::{Cli, Commands};

const RUN_FILE: &str = "/tmp/gif_walker.run";

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Stop) = cli.command {
        let _ = fs::remove_file(RUN_FILE);
        let mut stdout = io::stdout();
        let _ = terminal::clear_images(&mut stdout);
        println!("Stopped gif_walker.");
        return;
    }

    if cli.daemon {
        if let Err(e) = daemon::run(&cli, RUN_FILE) {
            eprintln!("Daemon error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Launch daemon
    if let Err(e) = fs::write(RUN_FILE, "1") {
        eprintln!("Failed to write run file {}: {}", RUN_FILE, e);
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
