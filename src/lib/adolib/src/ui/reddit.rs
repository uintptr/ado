use crate::{error::Result, llm::provider::LLMChain};

pub struct RedditQuery;

impl RedditQuery {
    pub fn new() -> RedditQuery {
        Self {}
    }

    pub async fn find_sub<S>(&self, chain: &LLMChain, desc: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        let msg = format!(
            "What is the sub reddit for {}. Just return the name of the subreddit starting with /r/ and nothing else",
            desc.as_ref()
        );

        chain.message(&msg).await
    }
}
