mod plugin;
mod prefetcher;
mod preloader;
mod previewer;

pub use plugin::*;
pub use prefetcher::*;
pub use preloader::*;
pub use previewer::*;

pub const MAX_PREWORKERS: u8 = 32;
