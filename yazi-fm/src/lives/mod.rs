#![allow(clippy::module_inception)]

mod active;
mod folder;
mod lives;
mod tabs;
mod tasks;

pub(super) use active::*;
pub(super) use folder::*;
pub(super) use lives::*;
pub(super) use tabs::*;
pub(super) use tasks::*;

type CtxRef<'lua> = mlua::UserDataRef<'lua, crate::Ctx>;
type FolderRef<'lua> = mlua::UserDataRef<'lua, yazi_core::folder::Folder>;
