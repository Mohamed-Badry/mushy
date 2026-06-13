

pub fn get_cell_dimensions() -> (u16, u16) {
    // SAFETY: `libc::winsize` is a C struct composed entirely of plain old data (integers).
    // It is always valid to represent it as all-zeros, so zero-initializing it is safe.
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    // Fallback estimates if terminal doesn't provide exact pixels
    let mut cell_width = 10;
    let mut cell_height = 20;

    // SAFETY: `ioctl` is unsafe because it operates on raw file descriptors and uses
    // raw pointers for out-parameters. We uphold the safety contract here because:
    // 1. `libc::STDOUT_FILENO` is a valid, standard file descriptor representing standard output.
    // 2. `TIOCGWINSZ` is the correct request code for getting the window size, which expects
    //    a pointer to a `winsize` struct and will write exactly `sizeof(struct winsize)` bytes.
    // 3. `&mut ws` is a mutable reference to a validly initialized `winsize` struct, providing
    //    a valid memory address and sufficient allocation for `ioctl` to write into.
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
