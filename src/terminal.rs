use std::io::{self, Write};

pub fn clear_images(stdout: &mut io::Stdout) -> io::Result<()> {
    write!(stdout, "\x1b_Ga=d,d=i,i=1,q=2\x1b\\")?;
    write!(stdout, "\x1b_Ga=d,d=i,i=2,q=2\x1b\\")?;
    stdout.flush()
}

pub fn get_cell_dimensions() -> (u16, u16) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    // Fallback estimates if terminal doesn't provide exact pixels
    let mut cell_width = 10;
    let mut cell_height = 20;

    unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 {
            if ws.ws_col > 0 && ws.ws_xpixel > 0 {
                cell_width = ws.ws_xpixel / ws.ws_col;
            }
            if ws.ws_row > 0 && ws.ws_ypixel > 0 {
                cell_height = ws.ws_ypixel / ws.ws_row;
            }
        }
    }

    (cell_width, cell_height)
}
