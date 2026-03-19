#[cfg(windows)]
pub(super) const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
pub(super) const SEPARATOR: char = std::path::MAIN_SEPARATOR;
