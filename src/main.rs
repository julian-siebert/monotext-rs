use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use monotext::Config;

#[derive(Debug, Parser)]
#[command(version, about, long_about)]
struct Cli {
    /// Input file (Markdown or XML)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output file (defaults to stdout if not provided)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Page height in lines (default: 50)
    #[arg(short = 'H', long, default_value_t = 50)]
    page_height: usize,

    /// Page width in characters (default: 70)
    #[arg(short = 'W', long, default_value_t = 70)]
    page_width: usize,

    /// Number of front-matter pages to use Roman numerals for (default: 2)
    #[arg(short, long, default_value_t = 2)]
    roman_pages: usize,

    /// Input format (Markdown or XML)
    #[arg(short, long, value_enum, default_value_t = InputFormat::Markdown)]
    format: InputFormat,
}

#[derive(Debug, Clone, ValueEnum)]
enum InputFormat {
    Markdown,
    Xml,
}

fn main() {
    let cli = Cli::parse();
    let cfg = Config {
        page_height: cli.page_height,
        page_width: cli.page_width,
        roman_pages: cli.roman_pages,
    };
}
