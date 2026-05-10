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

        /// Set the scale factor for the PDF
        #[arg(short, long, default_value_t = 1.0)]
        scale: f64,

        /// Generate an HTML file (Markdown -> HTML -> PDF). Useful for tweaking rendered PDF style.
        #[arg(long = "ghtml", default_value_t = false)]
        generate_html: bool,

        /// Allow raw HTML elements embedded in the Markdown source to pass through to the output.
        /// Note: this also permits unsafe link schemes such as javascript: and data:,
        /// as comrak controls both with a single flag. Only use this with Markdown you trust.
        #[arg(long = "allow-html", default_value_t = false)]
        allow_html: bool,

        /// Fit the entire document into a single PDF page. The page dimensions are
        /// measured from the rendered content, so nothing is clipped or scaled down.
        /// The template width is respected; only the height is adjusted to fit the content.
        /// Compatible with --scale: scale is applied first, then dimensions are measured.
        /// Mutually exclusive with --manual-breaks.
        #[arg(long = "one-page", default_value_t = false)]
        one_page: bool,

        /// Only break the page on explicit CSS break indicators in the content
        /// (e.g. <div style="break-after: page"></div>). Automatic page breaks
        /// inserted by Chrome based on the template page size are suppressed.
        /// The template @page size and margins are preserved unchanged.
        /// Mutually exclusive with --one-page.
        #[arg(long = "manual-breaks", default_value_t = false)]
        manual_breaks: bool,

        /// Add a custom metadata tag to the PDF (Title, Author, Subject, Keywords).
        /// Separate key from value with `=`.
        #[arg(long = "cm", action = clap::ArgAction::Append, num_args(1..))]
        custom_metadata: Vec<String>,
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