use std::collections::HashMap;

use crate::{
    error::{Error, Result},
    llm::gemini::{
        content::{ContentBuilder, Contents},
        genini::Gemini,
    },
};

use super::function_handler::FunctionTrait;

pub struct FunctionLlmGenerate {
    prompt: String,
}

impl FunctionLlmGenerate {
    #[cfg(test)]
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
        }
    }

    pub fn from_args(args: &HashMap<String, String>) -> Result<Self> {
        let prompt = match args.get("prompt") {
            Some(v) => v,
            None => {
                return Err(Error::MissingArgument {
                    name: "domain_name".to_string(),
                });
            }
        };

        Ok(Self {
            prompt: prompt.into(),
        })
    }
}

impl FunctionTrait for FunctionLlmGenerate {
    fn exec(&self) -> Result<String> {
        let mut contents = Contents::new();

        let content = ContentBuilder::new("user").with_text(&self.prompt).build();

        contents.with_content(&content);

        let g = Gemini::new()?;

        dbg!(&contents);

        let res = g.post_contents(contents)?;

        let text = res.get_text()?;

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm() {
        let llm = FunctionLlmGenerate::new("test");

        llm.exec().unwrap();
    }
}
