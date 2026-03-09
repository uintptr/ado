use std::fmt::Display;

use crate::data::types::AdoData;

pub trait ConsoleTrait {
    fn io(&self, data: AdoData) -> Option<String>;
    fn enter_thinking<M>(&self, message: M)
    where
        M: AsRef<str> + Display;
    fn leave_thinking(&self);
}
