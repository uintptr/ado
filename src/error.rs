pub type Result<T> = core::result::Result<T, Error>;

use std::path::PathBuf;

use derive_more::From;
use whois_rust::WhoIsError;

#[derive(Debug, From)]
pub enum Error {
    //
    // 1st party
    //
    DirnameError,
    QueryMissingError,
    FileNotFoundError {
        file_path: PathBuf,
    },
    InvalidInputType {
        input: String,
    },
    EmptyLlmResponse,
    EmptyLlmParts,
    LlmFunctionNotFound,
    LlmTextNotFound,
    UnknownFunction {
        name: String,
    },
    MissingArgument {
        name: String,
    },
    //
    // 2nd party
    //
    #[from]
    Io(std::io::Error),

    //
    // 3rd party
    //
    #[from]
    TomlDeserialize(toml::de::Error),
    #[from]
    JsonDeserialize(serde_json::Error),
    #[from]
    LoggingError(log::SetLoggerError),
    #[from]
    HttpError(minreq::Error),
    #[from]
    Whois(WhoIsError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
