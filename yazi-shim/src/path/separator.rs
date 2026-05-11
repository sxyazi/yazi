#[cfg(windows)]
pub const CROSS_SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
pub const CROSS_SEPARATOR: char = std::path::MAIN_SEPARATOR;
