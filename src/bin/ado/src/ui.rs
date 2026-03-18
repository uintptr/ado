use crossterm::terminal;

pub fn terminal_width() -> usize {
    terminal::size().map_or(80, |(w, _)| w as usize)
}
