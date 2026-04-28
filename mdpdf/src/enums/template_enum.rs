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
    TabletLight,
    /// A dark template for widescreen devices (laptops, desktops)
    WidescreenDark,
    /// A light template for widescreen devices (laptops, desktops)
    WidescreenLight,
    /// A dark template for widescreen devices (laptops, desktops) with a cutout for browser and taskbar UI elements
    WidescreenCutDark,
    /// A light template for widescreen devices (laptops, desktops) with a cutout for browser and taskbar UI elements
    WidescreenCutLight,
    /// A dark template for printing on A4 paper
    PrintA4Dark,
    /// A light template for printing on A4 paper
    PrintA4Light,
    /// A template for printing on white A4 paper
    PrintA4
}