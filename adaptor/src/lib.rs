#![allow(clippy::unit_arg)]

mod adaptor;
mod image;
mod iterm2;
mod kitty;
mod sixel;
mod ueberzug;

use iterm2::*;
use kitty::*;
use sixel::*;
use ueberzug::*;

pub use crate::{adaptor::*, image::*};
