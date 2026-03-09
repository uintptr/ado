use std::fmt::Display;

use crate::data::types::AdoData;

trait ConsoleTrait {
    fn io(&self, data: AdoData) -> Option<String>;
    fn toggle_spinner<M>(&self, message: M)
    where
        M: AsRef<str> + Display;
}
