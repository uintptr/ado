use std::fmt::Display;

use log::{error, info};
use serde::Serialize;

use crate::error::{Error, Result};

pub fn rest_get<S>(url: S) -> Result<String>
where
    S: AsRef<str> + Display,
{
    let mut res = ureq::get(url.as_ref()).call()?;

    let log_msg = format!(
        "get {url} -> code={} reason={}",
        res.status().as_u16(),
        res.status().as_str()
    );

    if res.status().is_success() {
        info!("{log_msg}");
    } else {
        error!("{log_msg}");
    }

    let resp_json = res.body_mut().read_to_string()?;

    Ok(resp_json)
}

pub fn rest_post<D, S>(url: S, data: D) -> Result<String>
where
    S: AsRef<str> + Display,
    D: Serialize,
{
    let mut res = ureq::post(url.as_ref())
        .header("Content-Type", "application/json")
        .send_json(data)?;

    let log_msg = format!(
        "post {url} -> code={} reason={}",
        res.status().as_u16(),
        res.status().as_str()
    );

    if res.status().is_success() {
        info!("{log_msg}");
    } else {
        error!("{log_msg}");
    }

    let resp_json = res.body_mut().read_to_string()?;

    Ok(resp_json)
}

pub fn rest_delete<S>(url: S) -> Result<()>
where
    S: AsRef<str> + Display,
{
    let res = ureq::delete(url.as_ref()).call()?;

    let log_msg = format!(
        "get {url} -> code={} reason={}",
        res.status().as_u16(),
        res.status().as_str()
    );

    if res.status().is_success() {
        info!("{log_msg}");
        Ok(())
    } else {
        error!("{log_msg}");
        Err(Error::HttpDeleteFailure)
    }
}
