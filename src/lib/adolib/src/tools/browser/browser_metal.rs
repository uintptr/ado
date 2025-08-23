use std::{path::PathBuf, process::Command};

use which::which;

use crate::{data::types::AdoData, error::Result, tools::args::ToolArgs};

pub struct FunctionsBrowser {
    xdg_open: PathBuf,
}

impl FunctionsBrowser {
    pub fn new() -> Result<Self> {
        let xdg_open = which("xdg-open")?;

        Ok(FunctionsBrowser { xdg_open })
    }

    pub fn browse(&self, args: &ToolArgs) -> Result<AdoData> {
        let url = args.get_string("url")?;

        let mut child = Command::new(&self.xdg_open).arg(url).spawn()?;

        let ret = child.wait()?;

        let ret_str = match ret.success() {
            true => "success",
            false => "failure",
        };

        Ok(AdoData::String(ret_str.to_string()))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn browse_url_test() {}
}
