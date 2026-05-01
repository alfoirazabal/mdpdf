use clap::{Parser, Subcommand};
use crate::enums::template_enum::Template;
use crate::constants;

#[derive(Parser)]
#[command(name = "mdpdf")]
#[command(author = constants::DEV_NAME, version = constants::VERSION, about = constants::about())]
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
        template: Option<Template>,

        /// Use a custom CSS template file instead of the built-in ones
        #[arg(short, long)]
        custom_template_path: Option<String>,
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