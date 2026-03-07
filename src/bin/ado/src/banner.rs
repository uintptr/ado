use std::path::PathBuf;

use anyhow::{Result, bail};
use figlet_rs::FIGfont;
use rust_embed::{Embed, EmbeddedFile};

#[derive(Embed)]
#[folder = "fonts"]
struct FigFonts;

pub fn display_banner<S, F>(text: S, font: F) -> Result<()>
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

    let fig_ret = match font_data {
        Some(v) => {
            let font_str = String::from_utf8_lossy(&v.data);
            FIGfont::from_content(&font_str)
        }
        None => FIGfont::standard(),
    };

    let fig = match fig_ret {
        Ok(v) => v,
        Err(e) => bail!("Unable to load fonts ({e})"),
    };

    let content = fig.convert(text.as_ref());

    if let Some(banner) = content {
        println!("{}", banner);
    }

    Ok(())
}
