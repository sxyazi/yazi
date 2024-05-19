mod plugin;
mod preloader;
mod previewer;

pub use plugin::*;
pub use preloader::*;
pub use previewer::*;

pub const MAX_PRELOADERS: u8 = 32;
