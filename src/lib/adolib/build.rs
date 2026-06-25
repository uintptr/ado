use core::result::Result;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let build = vergen_git2::Build::builder().build_date(true).build();
    let git = vergen_git2::Git2::builder().sha(false).build();
    vergen_git2::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&git)?
        .emit()?;
    Ok(())
}
