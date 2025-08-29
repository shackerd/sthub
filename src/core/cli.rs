use clap::{Parser, command};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub configuration_path: Option<String>,
}
