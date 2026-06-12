use std::fs;
use std::io::{self, Cursor, Read};

pub fn load_and_encode_frames(
    gif_path: Option<&str>,
    target_size: u32,
    cw: bool,
) -> io::Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
    let buffer = if let Some(path) = gif_path {
        match fs::File::open(path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                buf
            }
            Err(e) => {
                eprintln!("Failed to open gif {}, falling back to default: {}", path, e);
                include_bytes!("../mushroom.gif").to_vec()
            }
        }
    } else {
        include_bytes!("../mushroom.gif").to_vec()
    };

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

        let mut f_normal = Vec::new();
        small_dyn
            .write_to(
                &mut Cursor::new(&mut f_normal),
                image::ImageOutputFormat::Png,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut f_feet_right = Vec::new();
        small_dyn
            .rotate270()
            .write_to(
                &mut Cursor::new(&mut f_feet_right),
                image::ImageOutputFormat::Png,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut f_feet_top = Vec::new();
        small_dyn
            .rotate180()
            .write_to(
                &mut Cursor::new(&mut f_feet_top),
                image::ImageOutputFormat::Png,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut f_feet_left = Vec::new();
        small_dyn
            .rotate90()
            .write_to(
                &mut Cursor::new(&mut f_feet_left),
                image::ImageOutputFormat::Png,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

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

    Ok((frames_right, frames_up, frames_left, frames_down))
}
