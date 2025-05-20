use serde::{Deserialize, Serialize};

use crate::{
    config::AdoConfig,
    content::{Content, ContentBuilder, Contents, Part},
    error::{Error, Result},
    functions::AdoFunctions,
};

#[derive(Serialize)]
pub struct Gemini {
    functions: AdoFunctions,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Deserialize)]
pub struct GeminiUsage {
    #[serde(rename = "promptTokenCount")]
    pub token_count: i32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: i32,
    #[serde(rename = "totalTokenCount")]
    pub total_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage: GeminiUsage,
    #[serde(rename = "modelVersion")]
    pub model_version: String,
}

impl GeminiResponse {
    fn get_first_part(&self) -> Result<&Part> {
        let candidate = self.candidates.get(0).ok_or(Error::EmptyLlmResponse)?;

        if candidate.content.parts.is_empty() {
            return Err(Error::EmptyLlmParts);
        }

        match candidate.content.parts.get(0) {
            Some(v) => Ok(v),
            None => return Err(Error::EmptyLlmParts),
        }
    }

    pub fn call_function(&self) -> Result<(String, String)> {
        let part = self.get_first_part()?;

        match &part.function_call {
            Some(v) => {
                let resp = v.execute()?;
                Ok((v.name.to_string(), resp))
            }
            None => Err(Error::LlmFunctionNotFound),
        }
    }

    pub fn get_text(&self) -> Result<String> {
        let part = self.get_first_part()?;

        match &part.text {
            Some(v) => Ok(v.to_string()),
            None => Err(Error::LlmTextNotFound),
        }
    }
}

impl Gemini {
    pub fn new() -> Result<Self> {
        let functions = AdoFunctions::load()?;
        let config = AdoConfig::load()?;

        let g = config.gemini()?;

        let url = format!("{}{}", g.url, g.key);

        Ok(Gemini { functions, url })
    }

    fn parse_response(&self, response: &GeminiResponse) -> Result<GeminiResponse> {
        let candidate = response.candidates.get(0).ok_or(Error::EmptyLlmResponse)?;

        if candidate.content.parts.is_empty() {
            return Err(Error::EmptyLlmParts);
        }

        let mut contents = Contents::new(&self.functions);
        contents.with_content(&candidate.content);

        let (func_name, func_resp) = response.call_function()?;

        let resp_content = ContentBuilder::new("user")
            .with_response(func_name, func_resp)
            .build();

        contents.with_content(&resp_content);

        self.post_contents(contents)
    }

    fn post_contents(&self, contents: Contents) -> Result<GeminiResponse> {
        let json_content = contents.to_json()?;

        let res = minreq::post(&self.url)
            .with_header("Content-Type", "application/json")
            .with_body(json_content)
            .send()?;

        let response_json = res.as_str()?;
        let res: GeminiResponse = serde_json::from_str(response_json)?;

        Ok(res)
    }

    pub fn with_url<S>(&mut self, url: S)
    where
        S: AsRef<str>,
    {
        self.url = url.as_ref().to_string()
    }

    pub fn ask<S>(&self, query: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let mut contents = Contents::new(&self.functions);

        let cb = ContentBuilder::new("user").with_text(&query);

        let user_content = cb.build();
        contents.with_content(&user_content);

        let res = self.post_contents(contents)?;

        let res = self.parse_response(&res)?;

        res.get_text()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::staples::find_file;

    use super::*;

    #[test]
    fn test_de_resp() {
        let rel_test = Path::new("test").join("test_resp.json");

        let test_file = find_file(rel_test).unwrap();

        let resp_json = fs::read_to_string(test_file).unwrap();

        let gemini = Gemini::new().unwrap();

        let res: GeminiResponse = serde_json::from_str(&resp_json).unwrap();

        gemini.parse_response(&res).unwrap();
    }
}
