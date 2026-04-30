#[derive(Copy, Clone)]
pub enum StatusMessage {
    ReadingMarkdown,
    ConvertingToHtml,
    ApplyingTemplate,
    ConvertingHtmlToPdf,
    AttachingMdToPdf
}

pub trait MessageFetcher {
    fn get_message(&mut self, status: StatusMessage) -> &'static str;
}

pub struct StatusMessageProvider;

impl MessageFetcher for StatusMessageProvider {
    fn get_message(&mut self, status: StatusMessage) -> &'static str {
        match status {
            StatusMessage::ReadingMarkdown => "Reading Markdown...",
            StatusMessage::ConvertingToHtml => "Converting to HTML...",
            StatusMessage::ApplyingTemplate => "Applying template...",
            StatusMessage::ConvertingHtmlToPdf => "Converting HTML to PDF...",
            StatusMessage::AttachingMdToPdf => "Attaching MD to PDF...",
        }
    }
}