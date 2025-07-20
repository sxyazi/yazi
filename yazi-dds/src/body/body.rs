use anyhow::{Result, bail};
use mlua::{ExternalResult, IntoLua, Lua, Value};
use yazi_shared::Id;

use super::{BodyBulk, BodyBye, BodyCd, BodyCustom, BodyDelete, BodyHey, BodyHi, BodyHover, BodyLoad, BodyMount, BodyMove, BodyRename, BodyTab, BodyTrash, BodyYank};
use crate::Payload;

#[derive(Debug)]
pub enum Body<'a> {
	Hi(BodyHi<'a>),
	Hey(BodyHey),
	Bye(BodyBye),
	Tab(BodyTab),
	Cd(BodyCd<'a>),
	Load(BodyLoad<'a>),
	Hover(BodyHover<'a>),
	Rename(BodyRename<'a>),
	Bulk(BodyBulk<'a>),
	Yank(BodyYank<'a>),
	Move(BodyMove<'a>),
	Trash(BodyTrash<'a>),
	Delete(BodyDelete<'a>),
	Mount(BodyMount),
	Custom(BodyCustom),

	// Manager key events
	KeyArrow(&'a yazi_parser::ArrowOpt),
	KeyBack(&'a yazi_parser::VoidOpt),
	KeyBulkRename(&'a yazi_parser::VoidOpt),
	KeyCd(&'a yazi_parser::mgr::CdOpt),
	KeyClose(&'a yazi_parser::mgr::CloseOpt),
	KeyCopy(&'a yazi_parser::mgr::CopyOpt),
	KeyCreate(&'a yazi_parser::mgr::CreateOpt),
	KeyEnter(&'a yazi_parser::VoidOpt),
	KeyEscape(&'a yazi_parser::mgr::EscapeOpt),
	KeyEscapeFilter(&'a yazi_parser::VoidOpt),
	KeyEscapeFind(&'a yazi_parser::VoidOpt),
	KeyEscapeSearch(&'a yazi_parser::VoidOpt),
	KeyEscapeSelect(&'a yazi_parser::VoidOpt),
	KeyEscapeVisual(&'a yazi_parser::VoidOpt),
	KeyFilter(&'a yazi_parser::mgr::FilterOpt),
	KeyFilterDo(&'a yazi_parser::mgr::FilterOpt),
	KeyFind(&'a yazi_parser::mgr::FindOpt),
	KeyFindArrow(&'a yazi_parser::mgr::FindArrowOpt),
	KeyFindDo(&'a yazi_parser::mgr::FindDoOpt),
	KeyFollow(&'a yazi_parser::VoidOpt),
	KeyForward(&'a yazi_parser::VoidOpt),
	KeyHardlink(&'a yazi_parser::mgr::HardlinkOpt),
	KeyHidden(&'a yazi_parser::mgr::HiddenOpt),
	KeyHover(&'a yazi_parser::mgr::HoverOpt),
	KeyHoverDo(&'a yazi_parser::mgr::HoverDoOpt),
	KeyLeave(&'a yazi_parser::VoidOpt),
	KeyLinemode(&'a yazi_parser::mgr::LinemodeOpt),
	KeyLink(&'a yazi_parser::mgr::LinkOpt),
	KeyOpen(&'a yazi_parser::mgr::OpenOpt),
	KeyOpenDo(&'a yazi_parser::mgr::OpenDoOpt),
	KeyPaste(&'a yazi_parser::mgr::PasteOpt),
	KeyPeek(&'a yazi_parser::mgr::PeekOpt),
	KeyQuit(&'a yazi_parser::mgr::QuitOpt),
	KeyRefresh(&'a yazi_parser::VoidOpt),
	KeyRemove(&'a yazi_parser::mgr::RemoveOpt),
	KeyRemoveDo(&'a yazi_parser::mgr::RemoveOpt),
	KeyRename(&'a yazi_parser::mgr::RenameOpt),
	KeyReveal(&'a yazi_parser::mgr::RevealOpt),
	KeySearch(&'a yazi_parser::mgr::SearchOpt),
	KeySearchDo(&'a yazi_parser::mgr::SearchOpt),
	KeySearchStop(&'a yazi_parser::VoidOpt),
	KeySeek(&'a yazi_parser::mgr::SeekOpt),
	KeyShell(&'a yazi_parser::mgr::ShellOpt),
	KeySort(&'a yazi_parser::mgr::SortOpt),
	KeySpot(&'a yazi_parser::mgr::SpotOpt),
	KeySuspend(&'a yazi_parser::VoidOpt),
	KeyTabClose(&'a yazi_parser::mgr::TabCloseOpt),
	KeyTabCreate(&'a yazi_parser::mgr::TabCreateOpt),
	KeyTabSwap(&'a yazi_parser::ArrowOpt),
	KeyTabSwitch(&'a yazi_parser::mgr::TabSwitchOpt),
	KeyToggle(&'a yazi_parser::mgr::ToggleOpt),
	KeyToggleAll(&'a yazi_parser::mgr::ToggleAllOpt),
	KeyUnyank(&'a yazi_parser::VoidOpt),
	KeyUpdateFiles(&'a yazi_parser::mgr::UpdateFilesOpt),
	KeyUpdateMimes(&'a yazi_parser::mgr::UpdateMimesOpt),
	KeyUpdatePaged(&'a yazi_parser::mgr::UpdatePagedOpt),
	KeyUpdatePeeked(&'a yazi_parser::mgr::UpdatePeekedOpt),
	KeyUpdateSpotted(&'a yazi_parser::mgr::UpdateSpottedOpt),
	KeyUpdateTasks(&'a yazi_parser::mgr::UpdateTasksOpt),
	KeyUpdateYanked(&'a yazi_parser::mgr::UpdateYankedOpt<'a>),
	KeyVisualMode(&'a yazi_parser::mgr::VisualModeOpt),
	KeyWatch(&'a yazi_parser::VoidOpt),
	KeyYank(&'a yazi_parser::mgr::YankOpt),
	// Void
	Void(&'a yazi_parser::VoidOpt),
}

impl Body<'static> {
	pub fn from_str(kind: &str, body: &str) -> Result<Self> {
		Ok(match kind {
			"hi" => Self::Hi(serde_json::from_str(body)?),
			"hey" => Self::Hey(serde_json::from_str(body)?),
			"bye" => Self::Bye(serde_json::from_str(body)?),
			"tab" => Self::Tab(serde_json::from_str(body)?),
			"cd" => Self::Cd(serde_json::from_str(body)?),
			"load" => Self::Load(serde_json::from_str(body)?),
			"hover" => Self::Hover(serde_json::from_str(body)?),
			"rename" => Self::Rename(serde_json::from_str(body)?),
			"bulk" => Self::Bulk(serde_json::from_str(body)?),
			"@yank" => Self::Yank(serde_json::from_str(body)?),
			"move" => Self::Move(serde_json::from_str(body)?),
			"trash" => Self::Trash(serde_json::from_str(body)?),
			"delete" => Self::Delete(serde_json::from_str(body)?),
			"mount" => Self::Mount(serde_json::from_str(body)?),
			_ => BodyCustom::from_str(kind, body)?,
		})
	}

	pub fn from_lua(lua: &Lua, kind: &str, value: Value) -> mlua::Result<Self> {
		Self::validate(kind).into_lua_err()?;
		BodyCustom::from_lua(lua, kind, value)
	}

	pub fn validate(kind: &str) -> Result<()> {
		if matches!(
			kind,
			"hi"
				| "hey"
				| "bye"
				| "tab"
				| "cd"
				| "load"
				| "hover"
				| "rename"
				| "bulk"
				| "@yank"
				| "move"
				| "trash"
				| "delete"
				| "mount"
		) || kind.starts_with("emit-")
			|| kind.starts_with("emit-ind-")
			|| kind.starts_with("ind-")
			|| kind.starts_with("key-")
			|| kind.starts_with("relay-")
			|| kind.starts_with("relay-ind-")
		{
			bail!("Cannot construct system event");
		}

		let mut it = kind.bytes().peekable();
		if it.peek() == Some(&b'@') {
			it.next(); // Skip `@` as it's a prefix for static messages
		}
		if !it.all(|b| b.is_ascii_alphanumeric() || b == b'-') {
			bail!("Kind must be alphanumeric with dashes");
		}

		Ok(())
	}
}

impl<'a> Body<'a> {
	#[inline]
	pub fn kind(&self) -> &str {
		match self {
			Self::Hi(_) => "hi",
			Self::Hey(_) => "hey",
			Self::Bye(_) => "bye",
			Self::Tab(_) => "tab",
			Self::Cd(_) => "cd",
			Self::Load(_) => "load",
			Self::Hover(_) => "hover",
			Self::Rename(_) => "rename",
			Self::Bulk(_) => "bulk",
			Self::Yank(_) => "@yank",
			Self::Move(_) => "move",
			Self::Trash(_) => "trash",
			Self::Delete(_) => "delete",
			Self::Mount(_) => "mount",
			Self::Custom(b) => b.kind.as_str(),

			// Manager key events
			Self::KeyArrow(_) => "key-arrow",
			Self::KeyBack(_) => "key-back",
			Self::KeyBulkRename(_) => "key-bulk-rename",
			Self::KeyCd(_) => "key-cd",
			Self::KeyClose(_) => "key-close",
			Self::KeyCopy(_) => "key-copy",
			Self::KeyCreate(_) => "key-create",
			Self::KeyEnter(_) => "key-enter",
			Self::KeyEscape(_) => "key-escape",
			Self::KeyEscapeFilter(_) => "key-escape-filter",
			Self::KeyEscapeFind(_) => "key-escape-find",
			Self::KeyEscapeSearch(_) => "key-escape-search",
			Self::KeyEscapeSelect(_) => "key-escape-select",
			Self::KeyEscapeVisual(_) => "key-escape-visual",
			Self::KeyFilter(_) => "key-filter",
			Self::KeyFilterDo(_) => "key-filter-do",
			Self::KeyFind(_) => "key-find",
			Self::KeyFindArrow(_) => "key-find-arrow",
			Self::KeyFindDo(_) => "key-find-do",
			Self::KeyFollow(_) => "key-follow",
			Self::KeyForward(_) => "key-forward",
			Self::KeyHardlink(_) => "key-hardlink",
			Self::KeyHidden(_) => "key-hidden",
			Self::KeyHover(_) => "key-hover",
			Self::KeyHoverDo(_) => "key-hover-do",
			Self::KeyLeave(_) => "key-leave",
			Self::KeyLinemode(_) => "key-linemode",
			Self::KeyLink(_) => "key-link",
			Self::KeyOpen(_) => "key-open",
			Self::KeyOpenDo(_) => "key-open-do",
			Self::KeyPaste(_) => "key-paste",
			Self::KeyPeek(_) => "key-peek",
			Self::KeyQuit(_) => "key-quit",
			Self::KeyRefresh(_) => "key-refresh",
			Self::KeyRemove(_) => "key-remove",
			Self::KeyRemoveDo(_) => "key-remove-do",
			Self::KeyRename(_) => "key-rename",
			Self::KeyReveal(_) => "key-reveal",
			Self::KeySearch(_) => "key-search",
			Self::KeySearchDo(_) => "key-search-do",
			Self::KeySearchStop(_) => "key-search-stop",
			Self::KeySeek(_) => "key-seek",
			Self::KeyShell(_) => "key-shell",
			Self::KeySort(_) => "key-sort",
			Self::KeySpot(_) => "key-spot",
			Self::KeySuspend(_) => "key-suspend",
			Self::KeyTabClose(_) => "key-tab-close",
			Self::KeyTabCreate(_) => "key-tab-create",
			Self::KeyTabSwap(_) => "key-tab-swap",
			Self::KeyTabSwitch(_) => "key-tab-switch",
			Self::KeyToggle(_) => "key-toggle",
			Self::KeyToggleAll(_) => "key-toggle-all",
			Self::KeyUnyank(_) => "key-unyank",
			Self::KeyUpdateFiles(_) => "key-update-files",
			Self::KeyUpdateMimes(_) => "key-update-mimes",
			Self::KeyUpdatePaged(_) => "key-update-paged",
			Self::KeyUpdatePeeked(_) => "key-update-peeked",
			Self::KeyUpdateSpotted(_) => "key-update-spotted",
			Self::KeyUpdateTasks(_) => "key-update-tasks",
			Self::KeyUpdateYanked(_) => "key-update-yanked",
			Self::KeyVisualMode(_) => "key-visual-mode",
			Self::KeyWatch(_) => "key-watch",
			Self::KeyYank(_) => "key-yank",
			// Void
			Self::Void(_) => "void",
		}
	}

	#[inline]
	pub fn with_receiver(self, receiver: Id) -> Payload<'a> {
		Payload::new(self).with_receiver(receiver)
	}

	#[inline]
	pub fn with_sender(self, sender: Id) -> Payload<'a> { Payload::new(self).with_sender(sender) }
}

impl<'a> IntoLua for Body<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Hi(b) => b.into_lua(lua),
			Self::Hey(b) => b.into_lua(lua),
			Self::Bye(b) => b.into_lua(lua),
			Self::Cd(b) => b.into_lua(lua),
			Self::Load(b) => b.into_lua(lua),
			Self::Hover(b) => b.into_lua(lua),
			Self::Tab(b) => b.into_lua(lua),
			Self::Rename(b) => b.into_lua(lua),
			Self::Bulk(b) => b.into_lua(lua),
			Self::Yank(b) => b.into_lua(lua),
			Self::Move(b) => b.into_lua(lua),
			Self::Trash(b) => b.into_lua(lua),
			Self::Delete(b) => b.into_lua(lua),
			Self::Mount(b) => b.into_lua(lua),
			Self::Custom(b) => b.into_lua(lua),

			// Manager key events
			Self::KeyArrow(b) => b.into_lua(lua),
			Self::KeyBack(b) => b.into_lua(lua),
			Self::KeyBulkRename(b) => b.into_lua(lua),
			Self::KeyCd(b) => b.into_lua(lua),
			Self::KeyClose(b) => b.into_lua(lua),
			Self::KeyCopy(b) => b.into_lua(lua),
			Self::KeyCreate(b) => b.into_lua(lua),
			Self::KeyEnter(b) => b.into_lua(lua),
			Self::KeyEscape(b) => b.into_lua(lua),
			Self::KeyEscapeFilter(b) => b.into_lua(lua),
			Self::KeyEscapeFind(b) => b.into_lua(lua),
			Self::KeyEscapeSearch(b) => b.into_lua(lua),
			Self::KeyEscapeSelect(b) => b.into_lua(lua),
			Self::KeyEscapeVisual(b) => b.into_lua(lua),
			Self::KeyFilter(b) => b.into_lua(lua),
			Self::KeyFilterDo(b) => b.into_lua(lua),
			Self::KeyFind(b) => b.into_lua(lua),
			Self::KeyFindArrow(b) => b.into_lua(lua),
			Self::KeyFindDo(b) => b.into_lua(lua),
			Self::KeyFollow(b) => b.into_lua(lua),
			Self::KeyForward(b) => b.into_lua(lua),
			Self::KeyHardlink(b) => b.into_lua(lua),
			Self::KeyHidden(b) => b.into_lua(lua),
			Self::KeyHover(b) => b.into_lua(lua),
			Self::KeyHoverDo(b) => b.into_lua(lua),
			Self::KeyLeave(b) => b.into_lua(lua),
			Self::KeyLinemode(b) => b.into_lua(lua),
			Self::KeyLink(b) => b.into_lua(lua),
			Self::KeyOpen(b) => b.into_lua(lua),
			Self::KeyOpenDo(b) => b.into_lua(lua),
			Self::KeyPaste(b) => b.into_lua(lua),
			Self::KeyPeek(b) => b.into_lua(lua),
			Self::KeyQuit(b) => b.into_lua(lua),
			Self::KeyRefresh(b) => b.into_lua(lua),
			Self::KeyRemove(b) => b.into_lua(lua),
			Self::KeyRemoveDo(b) => b.into_lua(lua),
			Self::KeyRename(b) => b.into_lua(lua),
			Self::KeyReveal(b) => b.into_lua(lua),
			Self::KeySearch(b) => b.into_lua(lua),
			Self::KeySearchDo(b) => b.into_lua(lua),
			Self::KeySearchStop(b) => b.into_lua(lua),
			Self::KeySeek(b) => b.into_lua(lua),
			Self::KeyShell(b) => b.into_lua(lua),
			Self::KeySort(b) => b.into_lua(lua),
			Self::KeySpot(b) => b.into_lua(lua),
			Self::KeySuspend(b) => b.into_lua(lua),
			Self::KeyTabClose(b) => b.into_lua(lua),
			Self::KeyTabCreate(b) => b.into_lua(lua),
			Self::KeyTabSwap(b) => b.into_lua(lua),
			Self::KeyTabSwitch(b) => b.into_lua(lua),
			Self::KeyToggle(b) => b.into_lua(lua),
			Self::KeyToggleAll(b) => b.into_lua(lua),
			Self::KeyUnyank(b) => b.into_lua(lua),
			Self::KeyUpdateFiles(b) => b.into_lua(lua),
			Self::KeyUpdateMimes(b) => b.into_lua(lua),
			Self::KeyUpdatePaged(b) => b.into_lua(lua),
			Self::KeyUpdatePeeked(b) => b.into_lua(lua),
			Self::KeyUpdateSpotted(b) => b.into_lua(lua),
			Self::KeyUpdateTasks(b) => b.into_lua(lua),
			Self::KeyUpdateYanked(b) => b.into_lua(lua),
			Self::KeyVisualMode(b) => b.into_lua(lua),
			Self::KeyWatch(b) => b.into_lua(lua),
			Self::KeyYank(b) => b.into_lua(lua),
			// Void
			Self::Void(b) => b.into_lua(lua),
		}
	}
}
