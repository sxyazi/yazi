mod folder;
mod manager;
mod mode;
mod preview;
mod tab;
mod tabs;
mod watcher;

pub use folder::*;
pub use manager::*;
pub use mode::*;
pub use preview::*;
pub use tab::*;
pub use tabs::*;
pub use watcher::*;

pub const PARENT_RATIO: u32 = 1;
pub const CURRENT_RATIO: u32 = 4;
pub const PREVIEW_RATIO: u32 = 3;
pub const ALL_RATIO: u32 = PARENT_RATIO + CURRENT_RATIO + PREVIEW_RATIO;

pub const DIR_PADDING: u16 = 2;

pub const PREVIEW_BORDER: u16 = 2;
pub const PREVIEW_PADDING: u16 = 2;
