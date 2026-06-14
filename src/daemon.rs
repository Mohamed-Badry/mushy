use crossterm::cursor;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::fs;

use crate::cli::Cli;
use crate::config::Config;
use crate::terminal::get_cell_dimensions;
use crate::image_processing::{load_and_encode_frames, Direction};
struct TerminalBounds {
    term_cols: u16,
    term_rows: u16,
    left_col: u16,
    right_col: u16,
    top_row: u16,
    bottom_row: u16,
}

struct PetState {
    logical_col: f32,
    logical_row: f32,
    dir: Direction,
    frame_idx: usize,
    pid: u32,
    current_id: u32,
    z_index: u32,
}

impl PetState {
    fn new(pid: u32, pseudo_rand: u32, frames_len: usize) -> Self {
        Self {
            logical_col: 0.0,
            logical_row: 0.0,
            dir: Direction::Right,
            frame_idx: (pseudo_rand as usize) % frames_len,
            pid,
            current_id: pid * 2,
            z_index: 100 + (pseudo_rand % 1000),
        }
    }

    fn fast_forward(&mut self, bounds: &TerminalBounds, cw: bool, pseudo_rand: u32) {
        let width = bounds.right_col.saturating_sub(bounds.left_col);
        let height = bounds.bottom_row.saturating_sub(bounds.top_row);
        let perimeter = (width * 2 + height * 2).max(1);
        let offset_cells = pseudo_rand % perimeter as u32;

        self.logical_col = bounds.left_col as f32;
        self.logical_row = if cw { bounds.top_row as f32 } else { bounds.bottom_row as f32 };

        for _ in 0..offset_cells {
            self.advance(bounds, 1.0, 1.0, cw);
        }
    }

    fn advance(&mut self, bounds: &TerminalBounds, speed: f32, vertical_speed: f32, cw: bool) {
        match self.dir {
            Direction::Right => {
                self.logical_row = if cw { bounds.top_row as f32 } else { bounds.bottom_row as f32 };
                if self.logical_col >= bounds.right_col as f32 {
                    self.dir = if cw { Direction::Down } else { Direction::Up };
                    self.logical_col = bounds.right_col as f32;
                } else {
                    self.logical_col += speed;
                }
            }
            Direction::Up => {
                self.logical_col = if cw { bounds.left_col as f32 } else { bounds.right_col as f32 };
                if self.logical_row <= bounds.top_row as f32 {
                    self.dir = if cw { Direction::Right } else { Direction::Left };
                    self.logical_row = bounds.top_row as f32;
                } else {
                    self.logical_row -= vertical_speed;
                }
            }
            Direction::Left => {
                self.logical_row = if cw { bounds.bottom_row as f32 } else { bounds.top_row as f32 };
                if self.logical_col <= bounds.left_col as f32 {
                    self.dir = if cw { Direction::Up } else { Direction::Down };
                    self.logical_col = bounds.left_col as f32;
                } else {
                    self.logical_col -= speed;
                }
            }
            Direction::Down => {
                self.logical_col = if cw { bounds.right_col as f32 } else { bounds.left_col as f32 };
                if self.logical_row >= bounds.bottom_row as f32 {
                    self.dir = if cw { Direction::Left } else { Direction::Right };
                    self.logical_row = bounds.bottom_row as f32;
                } else {
                    self.logical_row += vertical_speed;
                }
            }
        }
    }
}

fn build_frame_payload(state: &PetState, bounds: &TerminalBounds, b64: &str) -> Option<Vec<u8>> {
    let mut col = state.logical_col.round() as u16;
    let mut row = state.logical_row.round() as u16;

    col = col.clamp(0, bounds.term_cols.saturating_sub(1));
    row = row.clamp(0, bounds.term_rows.saturating_sub(1));

    let next_id = if state.current_id == state.pid * 2 { state.pid * 2 + 1 } else { state.pid * 2 };
    let mut frame_buf = Vec::with_capacity(16384);

    if crossterm::queue!(frame_buf, cursor::SavePosition, cursor::MoveTo(col, row)).is_err() {
        return None;
    }

    let chunk_size = 4096;
    let chunks: Vec<&[u8]> = b64.as_bytes().chunks(chunk_size).collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let m = if i == chunks.len() - 1 { 0 } else { 1 };
        let chunk_str = std::str::from_utf8(chunk).ok()?;

        if i == 0 {
            write!(
                frame_buf,
                "\x1b_Gf=100,a=T,i={},z={},q=2,m={};{}\x1b\\",
                next_id, state.z_index, m, chunk_str
            ).ok()?;
        } else {
            write!(frame_buf, "\x1b_Gm={};{}\x1b\\", m, chunk_str).ok()?;
        }
    }

    write!(frame_buf, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", state.current_id).ok()?;
    crossterm::queue!(frame_buf, cursor::RestorePosition).ok()?;

    Some(frame_buf)
}

pub fn run(cli: &Cli) -> io::Result<()> {
    let run_file = cli.run_file.as_deref().unwrap_or("/tmp/gif_walker.unknown.run");
    let config = Config::load(&cli.config);

    let gif_path = cli.gif.clone().or(config.gif_path);
    let cw = cli.cw || config.rotate_clockwise.unwrap_or(false);
    let target_size = cli.size.or(config.target_size).unwrap_or(40);
    let speed = cli.speed.or(config.speed).unwrap_or(1.0);

    let (cell_width, cell_height) = get_cell_dimensions();
    let vertical_speed = speed * (cell_width as f32 / cell_height as f32);
    
    let img_cells_x = (target_size as f32 / cell_width as f32).ceil() as u16;
    let img_cells_y = (target_size as f32 / cell_height as f32).ceil() as u16;

    let margin_bottom: u16 = img_cells_y + 1;
    let margin_right: u16 = img_cells_x + 1;
    let margin_top: u16 = 0;
    let margin_left: u16 = 0;

    let frames = load_and_encode_frames(gif_path.as_deref(), target_size, cw)?;

    if frames.right.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout();
    let pid = std::process::id();
    let time_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
    let pseudo_rand = (pid as u128 ^ time_ms) as u32;

    let mut state = PetState::new(pid, pseudo_rand, frames.right.len());

    let mut bounds = TerminalBounds {
        term_cols: 80,
        term_rows: 24,
        left_col: margin_left,
        right_col: 80,
        top_row: margin_top,
        bottom_row: 24,
    };

    if let Ok(initial_size) = crossterm::terminal::size() {
        bounds.term_cols = initial_size.0;
        bounds.term_rows = initial_size.1;
        bounds.right_col = initial_size.0.saturating_sub(margin_right);
        bounds.bottom_row = initial_size.1.saturating_sub(margin_bottom);
    }

    state.fast_forward(&bounds, cw, pseudo_rand);

    loop {
        if !Path::new(run_file).exists() {
            break;
        }

        if let Ok(current_size) = crossterm::terminal::size() {
            bounds.term_cols = current_size.0;
            bounds.term_rows = current_size.1;
            bounds.right_col = current_size.0.saturating_sub(margin_right);
            bounds.bottom_row = current_size.1.saturating_sub(margin_bottom);
        }

        state.advance(&bounds, speed, vertical_speed, cw);

        let b64 = &frames.get(state.dir)[state.frame_idx];

        if let Some(payload) = build_frame_payload(&state, &bounds, b64) {
            let mut stdout_locked = stdout.lock();
            if stdout_locked.write_all(&payload).is_err() || stdout_locked.flush().is_err() {
                break;
            }
        } else {
            break;
        }

        state.current_id = if state.current_id == pid * 2 { pid * 2 + 1 } else { pid * 2 };
        state.frame_idx = (state.frame_idx + 1) % frames.right.len();
        
        thread::sleep(Duration::from_millis(50));
    }

    let _ = fs::remove_file(run_file);
    
    // Clean up graphics cleanly before exiting
    let _ = write!(stdout, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", pid * 2);
    let _ = write!(stdout, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", pid * 2 + 1);
    let _ = stdout.flush();

    Ok(())
}
