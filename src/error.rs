pub type Result<T> = core::result::Result<T, Error>;

use std::{env::VarError, path::PathBuf, string::FromUtf8Error};

use derive_more::From;

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
    ConfigNotFound,
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
    FunctionNotAvailable,
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
    CommandNotFound,
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
    Whois(whois_rust::WhoIsError),
    #[from]
    Base64Error(base64::DecodeError),
    #[from]
    Glob(glob::PatternError),
    #[from]
    X11(x11rb::errors::ConnectError),
    #[from]
    X11Connection(x11rb::errors::ConnectionError),
    #[from]
    X11Reply(x11rb::errors::ReplyError),
    #[from]
    Which(which::Error),
    #[from]
    ShellToken(shell_words::ParseError),
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
