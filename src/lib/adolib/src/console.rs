use crate::data::types::AdoData;

pub trait ConsoleTrait {
    fn io(&self, data: AdoData) -> Option<String>;
    fn error_message(&self, message: &str);
    fn print_markdown(&self, s: &str);
    fn enter_thinking(&self, message: &str);
    fn leave_thinking(&self);
}
