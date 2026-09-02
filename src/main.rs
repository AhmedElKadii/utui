fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("utui v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    color_eyre::install()?;
    utui::App::run()
}
