///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {

    use adolib::{config::loader::AdoConfig, data::types::AdoData, ui::commands::UserCommands};

    use crate::console::TerminalConsole;
    use adolib::ui::{ConsoleDisplayTrait, NopConsole};

    #[test]
    fn display_text() {
        let config = AdoConfig::from_default().unwrap();

        let command = UserCommands::new(&config).unwrap();
        let mut console = TerminalConsole::new(&command).unwrap();
        console.display(AdoData::String("Hello, World!".to_string())).unwrap();
    }

    #[tokio::test]
    async fn arg_parser() {
        let config = AdoConfig::from_default().unwrap();

        let mut cmd = UserCommands::new(&config).unwrap();

        let mut console = NopConsole {};

        cmd.handler("/quit", &mut console).await.unwrap();
    }
}
