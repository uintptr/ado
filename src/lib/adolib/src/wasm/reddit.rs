use crate::{config_file::loader::ConfigFile, error::Result, llm::openai::api::LLM};

pub struct RedditQuery {
    llm: LLM,
}

impl RedditQuery {
    pub fn new(config: &ConfigFile) -> Result<RedditQuery> {
        let llm = LLM::new(&config)?;

        Ok(RedditQuery { llm })
    }

    pub async fn find_sub<S>(&self, desc: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        //
        // TODO: check cache ?
        //

        let msg = format!(
            "What is the sub reddit for {}. Just return the name of the subreddit starting with /r/ and nothing else",
            desc.as_ref()
        );

        self.llm.message(msg).await

        //
        // TODO: update cache on success
        //
    }
}
