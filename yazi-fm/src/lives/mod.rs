#![allow(clippy::module_inception)]

mod config;
mod file;
mod files;
mod folder;
mod lives;
mod mode;
mod preview;
mod selected;
mod tab;
mod tabs;
mod tasks;
mod yanked;

use config::*;
use file::*;
use files::*;
use folder::*;
pub(super) use lives::*;
use mode::*;
use preview::*;
use selected::*;
use tab::*;
use tabs::*;
use tasks::*;
use yanked::*;

type CtxRef<'lua> = mlua::UserDataRef<'lua, crate::Ctx>;
