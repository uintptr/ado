pub fn question_detection<S>(query: S) -> bool
where
    S: AsRef<str>,
{
    let query = query.as_ref();

    if query.starts_with("how ")
        || query.starts_with("how's ")
        || query.starts_with("does ")
        || query.starts_with("is ")
        || query.starts_with("what ")
        || query.starts_with("what's")
        || query.starts_with("whats")
        || query.starts_with("is it")
        || query.starts_with("why ")
        || query.starts_with("which ")
        || query.starts_with("can ")
        || query.starts_with("if ")
        || query.starts_with("are there")
        || query.starts_with("where ")
        || query.starts_with("when ")
        || query.starts_with("are ")
        || query.starts_with("will ")
        || query.starts_with("should ")
        || query.starts_with("could ")
        || query.starts_with("would ")
        || query.starts_with("do ")
        || query.starts_with("have ")
        || query.starts_with("has ")
        || query.starts_with("who ")
        || query.starts_with("explain ")
        || query.starts_with("tell me ")
        || query.contains(" is it ")
        || query.contains(" how to ")
        || query.contains(" do you ")
        || query.contains(" can you ")
        || query.contains(" could you ")
        || query.contains(" would you ")
        || query.contains(" should I ")
        || query.ends_with("?")
    {
        return true;
    }

    false
}
