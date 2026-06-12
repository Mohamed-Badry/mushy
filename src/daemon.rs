use crossterm::cursor;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::cli::Cli;
use crate::config::Config;
use crate::image_processing::load_and_encode_frames;
use crate::terminal::{clear_images, get_cell_dimensions};

#[derive(Clone, Copy)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

pub fn run(cli: &Cli, run_file: &str) -> io::Result<()> {
    let config = Config::load(&cli.config);

    let gif_path = cli
        .gif
        .clone()
        .or(config.gif_path);

    let cw = if cli.cw {
        true
    } else {
        config.rotate_clockwise.unwrap_or(false)
    };

    let target_size = cli.size.or(config.target_size).unwrap_or(40);

    let (cell_width, cell_height) = get_cell_dimensions();
    let img_cells_x = (target_size as f32 / cell_width as f32).ceil() as u16;
    let img_cells_y = (target_size as f32 / cell_height as f32).ceil() as u16;

    let margin_bottom: u16 = img_cells_y + 1;
    let margin_right: u16 = img_cells_x + 2;
    let margin_top: u16 = 0;
    let margin_left: u16 = 0;

    let (frames_right, frames_up, frames_left, frames_down) =
        load_and_encode_frames(gif_path.as_deref(), target_size, cw)?;

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

    let _ = clear_images(&mut stdout);

    let mut frame_idx = 0;
    let mut current_id = 1;

    loop {
        if !Path::new(run_file).exists() {
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

        col = col.clamp(0, term_cols.saturating_sub(1));
        row = row.clamp(0, term_rows.saturating_sub(1));

        let b64 = match dir {
            Direction::Right => &frames_right[frame_idx],
            Direction::Up => &frames_up[frame_idx],
            Direction::Left => &frames_left[frame_idx],
            Direction::Down => &frames_down[frame_idx],
        };

        let next_id = if current_id == 1 { 2 } else { 1 };

        let mut frame_buf = Vec::with_capacity(16384);

        if crossterm::queue!(frame_buf, cursor::SavePosition, cursor::MoveTo(col, row)).is_err() {
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
                    frame_buf,
                    "\x1b_Gf=100,a=T,i={},q=2,m={};{}\x1b\\",
                    next_id, m, chunk_str
                )
                .is_err()
                {
                    write_failed = true;
                    break;
                }
            } else {
                if write!(frame_buf, "\x1b_Gm={};{}\x1b\\", m, chunk_str).is_err() {
                    write_failed = true;
                    break;
                }
            }
        }

        if write_failed {
            break;
        }

        if write!(frame_buf, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", current_id).is_err() {
            break;
        }

        if crossterm::queue!(frame_buf, cursor::RestorePosition).is_err() {
            break;
        }

        let mut stdout_locked = stdout.lock();
        if stdout_locked.write_all(&frame_buf).is_err() {
            break;
        }
        if stdout_locked.flush().is_err() {
            break;
        }

        current_id = next_id;
        frame_idx = (frame_idx + 1) % frames_right.len();
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
