use clap::{Parser, Subcommand};
use crossterm::{cursor, execute};
use serde::Deserialize;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

const RUN_FILE: &str = "/tmp/gif_walker.run";

#[derive(Deserialize, Debug, Default)]
struct Config {
    gif_path: Option<String>,
    rotate_clockwise: Option<bool>,
    margin_bottom: Option<u16>,
    margin_right: Option<u16>,
    margin_top: Option<u16>,
    margin_left: Option<u16>,
    target_size: Option<u32>,
}

#[derive(Clone, Copy)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A terminal pet that walks along the inner borders"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to a custom config.toml file
    #[arg(long, global = true)]
    config: Option<String>,

    /// Internal flag to run as daemon
    #[arg(long, hide = true)]
    daemon: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stop the currently running gif_walker daemon
    Stop,
}

fn clear_images(stdout: &mut io::Stdout) -> io::Result<()> {
    write!(stdout, "\x1b_Ga=d,d=i,i=1,q=2\x1b\\")?;
    write!(stdout, "\x1b_Ga=d,d=i,i=2,q=2\x1b\\")?;
    stdout.flush()
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Stop) = cli.command {
        let _ = fs::remove_file(RUN_FILE);
        let mut stdout = io::stdout();
        let _ = clear_images(&mut stdout);
        println!("Stopped gif_walker.");
        return;
    }

    if cli.daemon {
        if let Err(e) = run_daemon(cli.config) {
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
    if let Some(cp) = cli.config {
        cmd.arg("--config");
        cmd.arg(cp);
    }

    // detach from the current terminal so the user gets their prompt back
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

fn run_daemon(config_path_opt: Option<String>) -> io::Result<()> {
    let mut config = Config::default();

    let cfg_path = if let Some(cp) = config_path_opt {
        Some(PathBuf::from(cp))
    } else {
        directories::ProjectDirs::from("", "", "gif_walker")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
    };

    if let Some(cp) = cfg_path {
        if cp.exists() {
            if let Ok(contents) = fs::read_to_string(&cp) {
                if let Ok(parsed) = toml::from_str(&contents) {
                    config = parsed;
                } else {
                    eprintln!("Warning: Failed to parse config file at {:?}", cp);
                }
            }
        } else if let Some(parent) = cp.parent() {
            // Create default config if it doesn't exist
            let _ = fs::create_dir_all(parent);
            let default_toml = r#"
# Default configuration for gif_walker
# gif_path = "./mushroom.gif"
# rotate_clockwise = false
# target_size = 40
# margin_bottom = 2
# margin_right = 5
# margin_top = 0
# margin_left = 0
"#;
            let _ = fs::write(&cp, default_toml.trim_start());
        }
    }

    let gif_path = config
        .gif_path
        .unwrap_or_else(|| "./mushroom.gif".to_string());
    let cw = config.rotate_clockwise.unwrap_or(false);
    let target_size = config.target_size.unwrap_or(40);

    let margin_bottom = config.margin_bottom.unwrap_or(2);
    let margin_right = config.margin_right.unwrap_or(5);
    let margin_top = config.margin_top.unwrap_or(0);
    let margin_left = config.margin_left.unwrap_or(0);

    let mut file = fs::File::open(&gif_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder
        .read_info(Cursor::new(buffer))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut frames_right = Vec::new();
    let mut frames_up = Vec::new();
    let mut frames_left = Vec::new();
    let mut frames_down = Vec::new();

    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    {
        let img = image::RgbaImage::from_raw(
            frame.width as u32,
            frame.height as u32,
            frame.buffer.clone().into_owned(),
        )
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to create image from raw frame buffer",
            )
        })?;
        let dyn_img = image::DynamicImage::ImageRgba8(img);

        let small_dyn = dyn_img.resize_exact(
            target_size,
            target_size,
            image::imageops::FilterType::Nearest,
        );

        let f_normal = small_dyn.clone().into_rgba8().into_raw();
        let f_feet_right = small_dyn.rotate270().into_rgba8().into_raw();
        let f_feet_top = small_dyn.rotate180().into_rgba8().into_raw();
        let f_feet_left = small_dyn.rotate90().into_rgba8().into_raw();

        let (b_right, b_up, b_left, b_down);

        if cw {
            b_right = base64::encode(&f_feet_top);
            b_down = base64::encode(&f_feet_right);
            b_left = base64::encode(&f_normal);
            b_up = base64::encode(&f_feet_left);
        } else {
            b_right = base64::encode(&f_normal);
            b_up = base64::encode(&f_feet_right);
            b_left = base64::encode(&f_feet_top);
            b_down = base64::encode(&f_feet_left);
        }

        frames_right.push(b_right);
        frames_up.push(b_up);
        frames_left.push(b_left);
        frames_down.push(b_down);
    }

    if frames_right.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();

    let initial_size = crossterm::terminal::size().unwrap_or((80, 24));
    let term_rows = initial_size.1;

    let mut col: u16 = margin_left;
    let mut row: u16 = if cw {
        margin_top
    } else {
        term_rows.saturating_sub(margin_bottom)
    };
    let mut dir = Direction::Right;

    // Clean up any old instances
    let _ = clear_images(&mut stdout);

    let mut frame_idx = 0;
    let mut current_id = 1;

    loop {
        if !Path::new(RUN_FILE).exists() {
            let _ = clear_images(&mut stdout);
            break;
        }

        let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));

        let bottom_row = term_rows.saturating_sub(margin_bottom);
        let top_row = margin_top;
        let right_col = term_cols.saturating_sub(margin_right);
        let left_col = margin_left;

        match dir {
            Direction::Right => {
                row = if cw { top_row } else { bottom_row };
                if col >= right_col {
                    dir = if cw { Direction::Down } else { Direction::Up };
                } else {
                    col += 1;
                }
            }
            Direction::Up => {
                col = if cw { left_col } else { right_col };
                if row <= top_row {
                    dir = if cw {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                } else {
                    row = row.saturating_sub(1);
                }
            }
            Direction::Left => {
                row = if cw { bottom_row } else { top_row };
                if col <= left_col {
                    dir = if cw { Direction::Up } else { Direction::Down };
                } else {
                    col = col.saturating_sub(1);
                }
            }
            Direction::Down => {
                col = if cw { right_col } else { left_col };
                if row >= bottom_row {
                    dir = if cw {
                        Direction::Left
                    } else {
                        Direction::Right
                    };
                } else {
                    row += 1;
                }
            }
        }

        // Failsafe bounds
        col = col.clamp(0, term_cols.saturating_sub(1));
        row = row.clamp(0, term_rows.saturating_sub(1));

        let b64 = match dir {
            Direction::Right => &frames_right[frame_idx],
            Direction::Up => &frames_up[frame_idx],
            Direction::Left => &frames_left[frame_idx],
            Direction::Down => &frames_down[frame_idx],
        };

        let next_id = if current_id == 1 { 2 } else { 1 };

        if let Err(e) = execute!(stdout, cursor::SavePosition, cursor::MoveTo(col, row)) {
            eprintln!("Terminal error: {}", e);
            break;
        }

        let chunk_size = 4096;
        let chunks: Vec<&[u8]> = b64.as_bytes().chunks(chunk_size).collect();
        let mut write_failed = false;

        for (i, chunk) in chunks.iter().enumerate() {
            let m = if i == chunks.len() - 1 { 0 } else { 1 };
            let chunk_str = match std::str::from_utf8(chunk) {
                Ok(s) => s,
                Err(_) => {
                    write_failed = true;
                    break;
                }
            };

            if i == 0 {
                if write!(
                    stdout,
                    "\x1b_Gf=32,a=T,i={},q=2,s={},v={},m={};{}\x1b\\",
                    next_id, target_size, target_size, m, chunk_str
                )
                .is_err()
                {
                    write_failed = true;
                    break;
                }
            } else {
                if write!(stdout, "\x1b_Gm={};{}\x1b\\", m, chunk_str).is_err() {
                    write_failed = true;
                    break;
                }
            }
        }

        if write_failed {
            break;
        }

        // Wipe the old image from memory
        if write!(stdout, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", current_id).is_err() {
            break;
        }

        if execute!(stdout, cursor::RestorePosition).is_err() {
            break;
        }

        if stdout.flush().is_err() {
            break;
        }

        current_id = next_id;
        frame_idx = (frame_idx + 1) % frames_right.len();
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
