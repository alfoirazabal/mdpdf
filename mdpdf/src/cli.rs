use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "mdpdf")]
#[command(author = "Alfonso Irazabal Levy", version = "1.0", about = "MDPDF - Convert Markdown to PDF and vice versa - By Alfonso Irazabal Levy - V 1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Template {
    /// A dark template for mobile phones
    MobileDark,
    /// A light template for mobile phones
    MobileLight,
    /// A dark template for tablets
    TabletDark
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