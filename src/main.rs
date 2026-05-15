use monitaur_core::error::EngineResult;

fn main() -> EngineResult<()> {
    println!("Monitaur v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
