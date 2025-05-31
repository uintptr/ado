pub fn detect_shell_question(query: &str) -> bool {
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
        || query.contains(" is it ")
        || query.contains(" how to ")
    {
        return true;
    }

    false
}
