use std::{env, fs::File, io::{BufReader, Write}, thread, time::Duration};
use crossterm::{cursor, execute};
use gif::DecodeOptions;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <gif_file_path> | stop", args[0]);
        return Ok(());
    }

    if args[1] == "stop" {
        let _ = std::fs::remove_file("/tmp/gif_walker.run");
        let mut stdout = io::stdout();
        // Delete specifically the two image IDs we ping-pong between
        print!("\x1b_Ga=d,d=i,i=1,q=2\x1b\\");
        print!("\x1b_Ga=d,d=i,i=2,q=2\x1b\\");
        let _ = stdout.flush();
        println!("Gif walker stopped.");
        return Ok(());
    }

    if args.len() > 2 && args[2] == "--daemon" {
        run_daemon(&args[1])?;
    } else {
        let gif_path = &args[1];
        if !std::path::Path::new(gif_path).exists() {
            eprintln!("File not found: {}", gif_path);
            return Ok(());
        }

        File::create("/tmp/gif_walker.run")?;

        std::process::Command::new(env::current_exe()?)
            .arg(gif_path)
            .arg("--daemon")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::null())
            .spawn()?;
            
        println!("Gif walker started in background! Type `{} stop` to stop it.", args[0]);
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Right, // Bottom border
    Up,    // Right border
    Left,  // Top border
    Down,  // Left border
}

fn run_daemon(gif_path: &str) -> io::Result<()> {
    let file = File::open(gif_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let target_size = 40; 
    let mut stdout = io::stdout();

    let mut frames_right = Vec::new();
    let mut frames_up = Vec::new();
    let mut frames_left = Vec::new();
    let mut frames_down = Vec::new();

    while let Some(frame) = decoder.read_next_frame().map_err(|e| io::Error::new(io::ErrorKind::Other, e))? {
        let img = image::RgbaImage::from_raw(frame.width as u32, frame.height as u32, frame.buffer.clone().into_owned()).unwrap();
        let dyn_img = image::DynamicImage::ImageRgba8(img);
        
        let small = dyn_img.resize_exact(target_size, target_size, image::imageops::FilterType::Nearest);
        
        let right = small.clone().into_rgba8().into_raw();
        let up = small.rotate270().into_rgba8().into_raw();
        let left = small.rotate180().into_rgba8().into_raw();
        let down = small.rotate90().into_rgba8().into_raw();

        frames_right.push(base64::encode(&right));
        frames_up.push(base64::encode(&up));
        frames_left.push(base64::encode(&left));
        frames_down.push(base64::encode(&down));
    }

    if frames_right.is_empty() {
        return Ok(());
    }

    let (mut term_cols, mut term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let img_cols = 5;
    let img_rows = 2;

    let mut col: u16 = 0;
    let mut row: u16 = term_rows.saturating_sub(img_rows);
    let mut dir = Direction::Right;

    // Clean up any old instances
    print!("\x1b_Ga=d,d=i,i=1,q=2\x1b\\");
    print!("\x1b_Ga=d,d=i,i=2,q=2\x1b\\");
    let _ = stdout.flush();

    let mut frame_idx = 0;
    let mut current_id = 1;

    loop {
        if !std::path::Path::new("/tmp/gif_walker.run").exists() {
            print!("\x1b_Ga=d,d=i,i=1,q=2\x1b\\");
            print!("\x1b_Ga=d,d=i,i=2,q=2\x1b\\");
            let _ = stdout.flush();
            break;
        }

        let new_size = crossterm::terminal::size().unwrap_or((80, 24));
        term_cols = new_size.0;
        term_rows = new_size.1;

        match dir {
            Direction::Right => {
                if col + img_cols >= term_cols {
                    dir = Direction::Up;
                    row = row.saturating_sub(1);
                } else {
                    col += 1;
                }
            }
            Direction::Up => {
                if row == 0 {
                    dir = Direction::Left;
                    col = col.saturating_sub(1);
                } else {
                    row -= 1;
                }
            }
            Direction::Left => {
                if col == 0 {
                    dir = Direction::Down;
                    row += 1;
                } else {
                    col -= 1;
                }
            }
            Direction::Down => {
                if row + img_rows >= term_rows {
                    dir = Direction::Right;
                    col += 1;
                } else {
                    row += 1;
                }
            }
        }

        col = col.min(term_cols.saturating_sub(img_cols).max(0));
        row = row.min(term_rows.saturating_sub(img_rows).max(0));

        let b64 = match dir {
            Direction::Right => &frames_right[frame_idx],
            Direction::Up => &frames_up[frame_idx],
            Direction::Left => &frames_left[frame_idx],
            Direction::Down => &frames_down[frame_idx],
        };

        let next_id = if current_id == 1 { 2 } else { 1 };

        execute!(stdout, cursor::SavePosition, cursor::MoveTo(col, row))?;
        
        let chunk_size = 4096;
        let chunks: Vec<&[u8]> = b64.as_bytes().chunks(chunk_size).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            let m = if i == chunks.len() - 1 { 0 } else { 1 };
            let chunk_str = std::str::from_utf8(chunk).unwrap();
            if i == 0 {
                if let Err(_) = write!(stdout, "\x1b_Gf=32,a=T,i={},q=2,s={},v={},m={};{}\x1b\\", next_id, target_size, target_size, m, chunk_str) {
                    break;
                }
            } else {
                if let Err(_) = write!(stdout, "\x1b_Gm={};{}\x1b\\", m, chunk_str) {
                    break;
                }
            }
        }
        
        // Wipe the old image from memory (which cleanly kills its trail too)
        if let Err(_) = write!(stdout, "\x1b_Ga=d,d=i,i={},q=2\x1b\\", current_id) {
            break;
        }
        
        execute!(stdout, cursor::RestorePosition)?;
        
        if let Err(_) = stdout.flush() {
            break;
        }

        current_id = next_id;
        frame_idx = (frame_idx + 1) % frames_right.len();
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
