use apikey_vault_core::cli::{Cli, Commands};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match apikey_vault_core::cli::commands::execute(cli) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}
