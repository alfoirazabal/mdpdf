use clap::{ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum Template {
    /// A dark template for mobile phones
    MobileDark,
    /// A light template for mobile phones
    MobileLight,
    /// A dark template for tablets
    TabletDark,
    /// A light template for tablets
    TabletLight
}