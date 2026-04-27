use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mdpdf")]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Markdown to PDF
    Render {
        /// Input markdown file
        input: String,

        /// Output PDF file
        #[arg(short, long)]
        output: String,

        /// Template name
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Extract embedded Markdown from PDF
    Extract {
        /// Input PDF file
        input: String,

        /// Output markdown file
        #[arg(short, long)]
        output: String,
    },
}