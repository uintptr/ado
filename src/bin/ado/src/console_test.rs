///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {

    use std::{fs, path::Path};

    use adolib::{
        config::loader::AdoConfig, data::types::AdoData, logging::logger::setup_logger,
        search::google::GoogleSearchResults, shell::AdoShell, ui::commands::UserCommands,
    };

    use adolib::storage::persistent::PersistentStorage;

    use adolib::ui::{ConsoleDisplayTrait, NopConsole};

    // Tests using TerminalConsole require a real terminal (alternate screen, raw mode).
    // They are marked #[ignore] so they don't run in CI.

    #[ignore]
    #[test]
    fn display_text() {
        use crate::console::TerminalConsole;

        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();
        let command = UserCommands::new(&config, cache).unwrap();
        let mut console = TerminalConsole::new(&command).unwrap();
        console.display(AdoData::String("Hello, World!".to_string())).unwrap();
    }

    #[tokio::test]
    async fn arg_parser() {
        setup_logger(true).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let mut cmd = UserCommands::new(&config, cache).unwrap();

        let mut console = NopConsole {};

        cmd.handler("/quit", &mut console).await.unwrap();
    }

    #[ignore]
    #[test]
    fn display_search() {
        use crate::console::TerminalConsole;

        setup_logger(true).unwrap();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let json_file = Path::new(manifest_dir)
            .join("..")
            .join("..")
            .join("..")
            .join("test")
            .join("search_test.json")
            .canonicalize()
            .unwrap();

        let json_data = fs::read_to_string(json_file).unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();
        let command = UserCommands::new(&config, cache).unwrap();
        let mut console = TerminalConsole::new(&command).unwrap();

        let data = AdoData::SearchData(GoogleSearchResults { json_string: json_data });

        console.display(data).unwrap();
    }

    #[ignore]
    #[test]
    fn display_shell_test() {
        use crate::console::TerminalConsole;

        setup_logger(true).unwrap();

        let sh = AdoShell::new();

        let data = sh.exec("uname -a").unwrap();

        let config = AdoConfig::from_default().unwrap();

        let td = tempfile::Builder::new().prefix("console_test_").tempdir().unwrap();
        let cache_file = td.path().join("cache.db");
        let cache = PersistentStorage::from_path(cache_file).unwrap();

        let command = UserCommands::new(&config, cache).unwrap();

        let mut console = TerminalConsole::new(&command).unwrap();

        console.display(data).unwrap();
    }
}
