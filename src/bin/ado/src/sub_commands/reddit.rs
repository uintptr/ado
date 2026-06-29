use adolib::{
    cache::kv::KVCache, config::loader::AdoConfig, console::ConsoleTrait, llm::chain::LLMChain,
};
use anyhow::Result;
use log::{error, info};

use crate::commands::UserCommansTrait;

pub struct CommandReddit<'a> {
    model: String,
    cache: &'a KVCache,
}

const REDDIT_CACHE_REALM: &str = "reddit";

impl<'a> CommandReddit<'a> {
    #[must_use]
    pub fn new(config: &AdoConfig, cache: &'a KVCache) -> Self {
        Self {
            model: config.command().reddit.model.clone(),
            cache,
        }
    }

    fn query_cached<S: AsRef<str>>(&self, query: S) -> Option<String> {
        self.cache.get_string(REDDIT_CACHE_REALM, query)
    }

    fn query<S: AsRef<str>>(&self, query: S, chain: &LLMChain) -> Result<String> {
        if let Some(cached) = self.query_cached(&query) {
            info!("{} was cached", query.as_ref());
            return Ok(cached);
        }

        let ret = self.query_remote(&query, chain);

        if let Ok(data) = &ret
            && let Err(e) = self.cache.add(REDDIT_CACHE_REALM, query, data)
        {
            error!("unable to write cache entry ({e}");
        }

        ret
    }

    fn query_remote<S: AsRef<str>>(&self, input: S, chain: &LLMChain) -> Result<String> {
        let query = format!(
            "What is the sub reddit for {}. Just return the name of the subreddit starting with /r/ and nothing else",
            input.as_ref()
        );

        let ret = chain.message(query, Some(&self.model))?;

        Ok(ret)
    }
}

impl UserCommansTrait for CommandReddit<'_> {
    fn name(&self) -> &'static str {
        "reddit"
    }

    fn desc(&self) -> &'static str {
        "find a sub reddit"
    }

    fn callback(&mut self, input: &str, chain: &mut LLMChain, console: &dyn ConsoleTrait) {
        let Ok(ret) = self.query(input, chain) else {
            return;
        };

        console.print_line(&ret);
    }
}
