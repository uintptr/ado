use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Serializer;

pub fn base64_serializer<S>(bytes: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
}
