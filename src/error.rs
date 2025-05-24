pub type Result<T> = core::result::Result<T, Error>;

use std::{env::VarError, path::PathBuf, string::FromUtf8Error};

use derive_more::From;
use glob::PatternError;
use whois_rust::WhoIsError;
use x11rb::errors::{ConnectError, ReplyError};

#[derive(Debug, From)]
pub enum Error {
    //
    // 1st party
    //
    DirnameError,
    NotFound,
    InvalidFormat,
    Empty,
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
    NotImplemented,
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
    #[from]
    Utf8(FromUtf8Error),
    #[from]
    Env(VarError),

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
    #[from]
    X11(ConnectError),
    #[from]
    X11Connection(x11rb::errors::ConnectionError),
    #[from]
    X11Reply(ReplyError),
    #[from]
    Which(which::Error),
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
