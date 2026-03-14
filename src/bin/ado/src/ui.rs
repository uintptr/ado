use crossterm::terminal;

pub fn terminal_width() -> usize {
    terminal::size().map_or(80, |(w, _)| w as usize)
}

pub fn format_top_border(title: &str, width: usize) -> String {
    let title_section = if title.is_empty() {
        String::new()
    } else {
        format!(" {title} ")
    };
    // ┌ + ─ + title_section + ─*fill + ┐  =  width columns
    let fixed = 3; // ┌, leading ─, trailing ┐
    let fill = width.saturating_sub(fixed).saturating_sub(title_section.len());
    format!("┌─{title_section}{}┐", "─".repeat(fill))
}

pub fn format_bottom_border(width: usize) -> String {
    let inner = width.saturating_sub(2);
    format!("└{}┘", "─".repeat(inner))
}

pub fn to_u16(val: usize) -> u16 {
    u16::try_from(val).unwrap_or(u16::MAX)
}
