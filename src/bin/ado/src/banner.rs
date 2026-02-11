use std::path::PathBuf;

use figlet_rs::FIGfont;
use ratatui::{
    style::{Color, Style},
    text::Text,
};
use rust_embed::{Embed, EmbeddedFile};

#[derive(Embed)]
#[folder = "fonts"]
struct FigFonts;

pub fn generate_banner<S, F>(text: S, font: F) -> Text<'static>
where
    S: AsRef<str>,
    F: AsRef<str>,
{
    let mut font_data: Option<EmbeddedFile> = None;

    for efont in FigFonts::iter() {
        let file_ext = PathBuf::from(efont.to_string());

        if let Some(font_name) = file_ext.file_stem()
            && font.as_ref() == font_name
        {
            font_data = FigFonts::get(&efont);
        }
    }

    let fig = match font_data {
        Some(v) => {
            let font_str = String::from_utf8_lossy(&v.data);
            match FIGfont::from_content(&font_str) {
                Ok(f) => f,
                Err(_) => return Text::raw(""),
            }
        }
        None => match FIGfont::standard() {
            Ok(f) => f,
            Err(_) => return Text::raw(""),
        },
    };

    let content = fig.convert(text.as_ref());

    match content {
        Some(banner) => {
            let banner_str = banner.to_string();
            Text::styled(banner_str, Style::default().fg(Color::Cyan))
        }
        None => Text::raw(""),
    }
}
