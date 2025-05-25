use std::collections::HashSet;

use log::info;
use serde::Serialize;
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt},
    rust_connection::RustConnection,
};

use crate::error::{Error, Result};

#[derive(Debug, Serialize, Hash, PartialEq, Eq)]
pub struct Window {
    name: String,
    pid: u32,
}

#[derive(Debug)]
pub struct FunctionsDesktop {
    conn: RustConnection,
    screen_num: usize,
}

impl FunctionsDesktop {
    pub fn new() -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None)?;

        Ok(Self { conn, screen_num })
    }

    fn name_from_windows_id(&self, window_id: u32) -> Result<String> {
        let prop = self
            .conn
            .get_property(
                false,
                window_id,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                0,
                u32::MAX,
            )?
            .reply()?;

        if prop.value.is_empty() {
            return Err(Error::Empty);
        }

        let name = String::from_utf8(prop.value)?;

        Ok(name)
    }

    fn pid_from_windows_id(&self, window_id: u32) -> Result<u32> {
        let pid_atom = self.conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;

        let prop = self
            .conn
            .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;

        if 32 != prop.format || 4 != prop.value.len() {
            return Err(Error::InvalidFormat);
        }

        let pid = u32::from_ne_bytes([prop.value[0], prop.value[1], prop.value[2], prop.value[3]]);
        Ok(pid)
    }

    pub fn windows_list(&self) -> Result<HashSet<Window>> {
        let mut windows = HashSet::new();

        let screen = &self.conn.setup().roots[self.screen_num];

        let root = screen.root;

        let reply = self.conn.query_tree(root)?.reply()?;

        for window_id in reply.children {
            let name = match self.name_from_windows_id(window_id) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let pid = self.pid_from_windows_id(window_id).unwrap_or(0);
            let w = Window { name, pid };

            windows.insert(w);
        }

        info!("found {} windows", windows.len());

        Ok(windows)
    }

    pub fn windows(&self) -> Result<String> {
        let windows = self.windows_list()?;

        let windows_string = serde_json::to_string_pretty(&windows)?;

        Ok(windows_string)
    }
}

mod tests {

    #[test]
    fn x11_list_windows() {
        use crate::functions::functions_desktop_x11::FunctionsDesktop;
        use crate::staples::setup_logger;

        setup_logger(true).unwrap();

        let desktop = FunctionsDesktop::new().unwrap();
        let s = desktop.windows().unwrap();
        assert!(0 != s.len());
    }
}
