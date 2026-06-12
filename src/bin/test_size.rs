use crossterm::terminal;
fn main() {
    println!("{:?}", terminal::window_size());
}
