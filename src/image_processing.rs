use std::fs;
use std::io::{self, Cursor, Read};

fn encode_rotated(img: &image::DynamicImage, angle: u32) -> io::Result<String> {
    let rotated = match angle {
        0 => img.clone(),
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => unreachable!(),
    };
    let mut buf = Vec::new();
    rotated
        .write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(base64::encode(&buf))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

pub struct PetFrames {
    pub right: Vec<String>,
    pub up: Vec<String>,
    pub left: Vec<String>,
    pub down: Vec<String>,
}

impl PetFrames {
    pub fn get(&self, dir: Direction) -> &Vec<String> {
        match dir {
            Direction::Right => &self.right,
            Direction::Up => &self.up,
            Direction::Left => &self.left,
            Direction::Down => &self.down,
        }
    }
}

pub fn load_and_encode_frames(
    gif_path: Option<&str>,
    target_size: u32,
    cw: bool,
) -> io::Result<PetFrames> {
    const DEFAULT_GIF_BYTES: &[u8] = include_bytes!("../mushroom.gif");

    let buffer = if let Some(path) = gif_path {
        match fs::File::open(path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                buf
            }
            Err(e) => {
                eprintln!("Failed to open gif {}, falling back to default: {}", path, e);
                DEFAULT_GIF_BYTES.to_vec()
            }
        }
    } else {
        DEFAULT_GIF_BYTES.to_vec()
    };

    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder
        .read_info(Cursor::new(buffer))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut right = Vec::new();
    let mut up = Vec::new();
    let mut left = Vec::new();
    let mut down = Vec::new();

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

        let angles = if cw {
            [
                (Direction::Right, 180),
                (Direction::Up, 90),
                (Direction::Left, 0),
                (Direction::Down, 270),
            ]
        } else {
            [
                (Direction::Right, 0),
                (Direction::Up, 270),
                (Direction::Left, 180),
                (Direction::Down, 90),
            ]
        };

        for (dir, angle) in angles {
            let encoded = encode_rotated(&small_dyn, angle)?;
            match dir {
                Direction::Right => right.push(encoded),
                Direction::Up => up.push(encoded),
                Direction::Left => left.push(encoded),
                Direction::Down => down.push(encoded),
            }
        }
    }

    Ok(PetFrames { right, up, left, down })
}
