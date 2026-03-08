pub type Result<T> = core::result::Result<T, Error>;

use std::{env::VarError, path::PathBuf, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    //
    // 1st party
    //
    #[error("EOF")]
    EOF,
    #[error("ResetInput")]
    ResetInput,
    #[error("DirnameError")]
    DirnameError,
    #[error("NotFound")]
    NotFound,
    #[error("InvalidFormat")]
    InvalidFormat,
    #[error("NotInitialized")]
    NotInitialized,
    #[error("Empty")]
    Empty,
    #[error("EmptySearchResult")]
    EmptySearchResult,
    #[error("InvalidJsonType")]
    InvalidJsonType,
    #[error("HttpGetFailure")]
    HttpGetFailure,
    #[error("QueryMissingError")]
    QueryMissingError,
    #[error("ConfigNotFound")]
    ConfigNotFound,
    #[error("FileNotFoundError: {file_path:?}")]
    FileNotFoundError { file_path: PathBuf },
    #[error("FileTooLarge: size={size}, limit={limit}")]
    FileTooLarge { size: u64, limit: u64 },
    #[error("HomeDirNotFound")]
    HomeDirNotFound,
    #[error("InvalidInputType: {input}")]
    InvalidInputType { input: String },
    #[error("EmptyLlmResponse")]
    EmptyLlmResponse,
    #[error("EmptyLlmParts")]
    EmptyLlmParts,
    #[error("LlmFunctionNotFound")]
    LlmFunctionNotFound,
    #[error("LlmTextNotFound")]
    LlmTextNotFound,
    #[error("LlmError: {message}")]
    LlmError { message: String },
    #[error("NotImplemented")]
    NotImplemented,
    #[error("FunctionNotImplemented: {name}")]
    FunctionNotImplemented { name: String },
    #[error("FunctionNotSupported")]
    FunctionNotSupported,
    #[error("FunctionNotAvailable")]
    FunctionNotAvailable,
    #[error("UnknownFunction: {name}")]
    UnknownFunction { name: String },
    #[error("InvalidFilePath: {path:?}")]
    InvalidFilePath { path: PathBuf },
    #[error("TypeError: {error}")]
    TypeError { error: String },
    #[error("TypeMissing: {t}")]
    TypeMissing { t: String },
    #[error("ContentTextNotFound")]
    ContentTextNotFound,
    #[error("LlmNotFound: {llm}")]
    LlmNotFound { llm: String },
    #[error("MissingArgument: {name}")]
    MissingArgument { name: String },
    #[error("API was not found. Either use an config file or define the OPENAI_API_KEY env variable")]
    ApiKeyNotFound,
    #[error("ApiFailure: {message}")]
    ApiFailure { message: String },
    #[error("CommandNotFound: {command}")]
    CommandNotFound { command: String },
    #[error("Usage: {help}")]
    Usage { help: String },
    #[error("StorageWriteFailure")]
    StorageWriteFailure,
    #[error("ConfigError: {error}")]
    ConfigError { error: String },
    #[error("ToolNotFound")]
    ToolNotFound,
    //
    // 2nd party
    //
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Env(#[from] VarError),
    #[error("{0}")]
    StrError(String),

    //
    // 3rd party
    //
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error(transparent)]
    JsonDeserialize(#[from] serde_json::Error),
    #[error(transparent)]
    LoggingError(#[from] log::SetLoggerError),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    Glob(#[from] glob::PatternError),
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
    #[error(transparent)]
    Http(#[from] ureq::Error),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::StrError(s)
    }
}
