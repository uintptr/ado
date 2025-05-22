pub type Result<T> = core::result::Result<T, Error>;

use std::path::PathBuf;

use derive_more::From;
use glob::PatternError;
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
    FunctionNotImplemented {
        name: String,
    },
    UnknownFunction {
        name: String,
    },
    InvalidFilePath {
        path: PathBuf,
    },
    TypeError {
        error: String,
    },
    TypeMissing {
        t: String,
    },
    ContentTextNotFound,
    LlmNotFound {
        llm: String,
    },
    MissingArgument {
        name: String,
    },
    ApiKeyNotFound,
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
    #[from]
    Base64Error(base64::DecodeError),
    #[from]
    Glob(PatternError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        match self {
            Error::ApiKeyNotFound => write!(
                fmt,
                "API was not found. Either use an config file or define the OPENAI_API_KEY env variable"
            ),
            _ => write!(fmt, "{self:?}"),
        }
    }
}
