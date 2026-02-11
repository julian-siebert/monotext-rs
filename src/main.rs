use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use monotext::{Config, Document, md::markdown_to_document, pdf::write_pdf};

#[derive(Debug, Parser)]
#[command(version, about, long_about)]
struct Cli {
    /// Input file (Markdown or XML)
    input: PathBuf,

    /// Output file (defaults to stdout if not provided)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Page height in lines (default: 50)
    #[arg(short = 'H', long, default_value_t = 57)]
    page_height: usize,

    /// Page width in characters (default: 70)
    #[arg(short = 'W', long, default_value_t = 70)]
    page_width: usize,

    /// Input format
    #[arg(short = 'I', long, value_enum)]
    input_format: Option<InputFormat>,

    /// Output format
    #[arg(short = 'O', long, value_enum)]
    output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, ValueEnum)]
enum InputFormat {
    Markdown,
    Xml,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Pdf,
    Html,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config {
        page_height: cli.page_height,
        page_width: cli.page_width,
    };

    let input_text = fs::read_to_string(&cli.input)?;

    let format = cli
        .input_format
        .or_else(|| match cli.input.extension().and_then(|s| s.to_str()) {
            Some("md") => Some(InputFormat::Markdown),
            Some("xml") => Some(InputFormat::Xml),
            _ => None,
        })
        .unwrap_or(InputFormat::Markdown);

    let document: Document = match format {
        InputFormat::Markdown => markdown_to_document(&input_text)?,
        InputFormat::Xml => anyhow::bail!("XML parser not implemented yet"),
    };

    let output_format = cli.output_format.unwrap_or(OutputFormat::Text);

    match output_format {
        OutputFormat::Text => {
            let rendered = document.render(cfg);

            match cli.output {
                Some(path) => fs::write(path, rendered)?,
                None => io::stdout().write_all(rendered.as_bytes())?,
            };
        }
        OutputFormat::Pdf => {
            let pdf_bytes = write_pdf(cfg, document)?;

            match cli.output {
                Some(path) => fs::write(path, pdf_bytes)?,
                None => io::stdout().write_all(&pdf_bytes)?,
            };
        }
        OutputFormat::Html => {}
    }

    Ok(())
}
