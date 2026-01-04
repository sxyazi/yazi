use mlua::{FromLua, IntoLua, Lua, Value};

use crate::{spark::SparkKind, try_from_spark};

#[derive(Debug)]
pub enum Spark<'a> {
	// Void
	Void(yazi_parser::VoidOpt),

	// Mgr
	Arrow(yazi_parser::ArrowOpt),
	Back(yazi_parser::VoidOpt),
	BulkRename(yazi_parser::VoidOpt),
	Cd(yazi_parser::mgr::CdOpt),
	Close(yazi_parser::mgr::CloseOpt),
	Copy(yazi_parser::mgr::CopyOpt),
	Create(yazi_parser::mgr::CreateOpt),
	Displace(yazi_parser::VoidOpt),
	DisplaceDo(yazi_parser::mgr::DisplaceDoOpt),
	Download(yazi_parser::mgr::DownloadOpt),
	Enter(yazi_parser::VoidOpt),
	Escape(yazi_parser::mgr::EscapeOpt),
	EscapeFilter(yazi_parser::VoidOpt),
	EscapeFind(yazi_parser::VoidOpt),
	EscapeSearch(yazi_parser::VoidOpt),
	EscapeSelect(yazi_parser::VoidOpt),
	EscapeVisual(yazi_parser::VoidOpt),
	Filter(yazi_parser::mgr::FilterOpt),
	FilterDo(yazi_parser::mgr::FilterOpt),
	Find(yazi_parser::mgr::FindOpt),
	FindArrow(yazi_parser::mgr::FindArrowOpt),
	FindDo(yazi_parser::mgr::FindDoOpt),
	Follow(yazi_parser::VoidOpt),
	Forward(yazi_parser::VoidOpt),
	Hardlink(yazi_parser::mgr::HardlinkOpt),
	Hidden(yazi_parser::mgr::HiddenOpt),
	Hover(yazi_parser::mgr::HoverOpt),
	Leave(yazi_parser::VoidOpt),
	Linemode(yazi_parser::mgr::LinemodeOpt),
	Link(yazi_parser::mgr::LinkOpt),
	Open(yazi_parser::mgr::OpenOpt),
	OpenDo(yazi_parser::mgr::OpenDoOpt),
	Paste(yazi_parser::mgr::PasteOpt),
	Peek(yazi_parser::mgr::PeekOpt),
	Quit(yazi_parser::mgr::QuitOpt),
	Refresh(yazi_parser::VoidOpt),
	Remove(yazi_parser::mgr::RemoveOpt),
	RemoveDo(yazi_parser::mgr::RemoveOpt),
	Rename(yazi_parser::mgr::RenameOpt),
	Reveal(yazi_parser::mgr::RevealOpt),
	Search(yazi_parser::mgr::SearchOpt),
	SearchDo(yazi_parser::mgr::SearchOpt),
	SearchStop(yazi_parser::VoidOpt),
	Seek(yazi_parser::mgr::SeekOpt),
	Shell(yazi_parser::mgr::ShellOpt),
	Sort(yazi_parser::mgr::SortOpt),
	Spot(yazi_parser::mgr::SpotOpt),
	Stash(yazi_parser::mgr::StashOpt),
	Suspend(yazi_parser::VoidOpt),
	TabClose(yazi_parser::mgr::TabCloseOpt),
	TabCreate(yazi_parser::mgr::TabCreateOpt),
	TabSwap(yazi_parser::ArrowOpt),
	TabSwitch(yazi_parser::mgr::TabSwitchOpt),
	Toggle(yazi_parser::mgr::ToggleOpt),
	ToggleAll(yazi_parser::mgr::ToggleAllOpt),
	Unyank(yazi_parser::VoidOpt),
	UpdateFiles(yazi_parser::mgr::UpdateFilesOpt),
	UpdateMimes(yazi_parser::mgr::UpdateMimesOpt),
	UpdatePaged(yazi_parser::mgr::UpdatePagedOpt),
	UpdatePeeked(yazi_parser::mgr::UpdatePeekedOpt),
	UpdateSpotted(yazi_parser::mgr::UpdateSpottedOpt),
	UpdateYanked(yazi_parser::mgr::UpdateYankedOpt<'a>),
	Upload(yazi_parser::mgr::UploadOpt),
	VisualMode(yazi_parser::mgr::VisualModeOpt),
	Watch(yazi_parser::VoidOpt),
	Yank(yazi_parser::mgr::YankOpt),

	// Cmp
	CmpArrow(yazi_parser::ArrowOpt),
	CmpClose(yazi_parser::cmp::CloseOpt),
	CmpShow(yazi_parser::cmp::ShowOpt),
	CmpTrigger(yazi_parser::cmp::TriggerOpt),

	// Confirm
	ConfirmArrow(yazi_parser::ArrowOpt),
	ConfirmClose(yazi_parser::confirm::CloseOpt),
	ConfirmShow(Box<yazi_parser::confirm::ShowOpt>),

	// Help
	HelpArrow(yazi_parser::ArrowOpt),
	HelpEscape(yazi_parser::VoidOpt),
	HelpFilter(yazi_parser::VoidOpt),
	HelpToggle(yazi_parser::help::ToggleOpt),

	// Input
	InputBackspace(yazi_parser::input::BackspaceOpt),
	InputBackward(yazi_parser::input::BackwardOpt),
	InputClose(yazi_parser::input::CloseOpt),
	InputComplete(yazi_parser::input::CompleteOpt),
	InputDelete(yazi_parser::input::DeleteOpt),
	InputEscape(yazi_parser::VoidOpt),
	InputForward(yazi_parser::input::ForwardOpt),
	InputInsert(yazi_parser::input::InsertOpt),
	InputKill(yazi_parser::input::KillOpt),
	InputMove(yazi_parser::input::MoveOpt),
	InputPaste(yazi_parser::input::PasteOpt),
	InputShow(yazi_parser::input::ShowOpt),

	// Notify
	NotifyTick(yazi_parser::notify::TickOpt),

	// Pick
	PickArrow(yazi_parser::ArrowOpt),
	PickClose(yazi_parser::pick::CloseOpt),
	PickShow(yazi_parser::pick::ShowOpt),

	// Spot
	SpotArrow(yazi_parser::ArrowOpt),
	SpotClose(yazi_parser::VoidOpt),
	SpotCopy(yazi_parser::spot::CopyOpt),
	SpotSwipe(yazi_parser::ArrowOpt),

	// Tasks
	TasksArrow(yazi_parser::ArrowOpt),
	TasksCancel(yazi_parser::VoidOpt),
	TasksClose(yazi_parser::VoidOpt),
	TasksInspect(yazi_parser::VoidOpt),
	TasksOpenShellCompat(yazi_parser::tasks::ProcessOpenOpt),
	TasksProcessOpen(yazi_parser::tasks::ProcessOpenOpt),
	TasksShow(yazi_parser::VoidOpt),
	TasksUpdateSucceed(yazi_parser::tasks::UpdateSucceedOpt),

	// Which
	WhichCallback(yazi_parser::which::CallbackOpt),
	WhichShow(yazi_parser::which::ShowOpt),
}

impl<'a> Spark<'a> {
	pub fn from_lua(lua: &Lua, kind: SparkKind, value: Value) -> mlua::Result<Self> {
		use SparkKind::*;

		Ok(match kind {
			// Sort
			KeySort => Self::Sort(<_>::from_lua(value, lua)?),
			IndSort => Self::Sort(<_>::from_lua(value, lua)?),
			// Stash
			IndStash => Self::Stash(<_>::from_lua(value, lua)?),
			RelayStash => Self::Stash(<_>::from_lua(value, lua)?),
			// Quit
			KeyQuit => Self::Quit(<_>::from_lua(value, lua)?),
		})
	}
}

impl<'a> IntoLua for Spark<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			// Void
			Self::Void(b) => b.into_lua(lua),

			// Mgr
			Self::Arrow(b) => b.into_lua(lua),
			Self::Back(b) => b.into_lua(lua),
			Self::BulkRename(b) => b.into_lua(lua),
			Self::Cd(b) => b.into_lua(lua),
			Self::Close(b) => b.into_lua(lua),
			Self::Copy(b) => b.into_lua(lua),
			Self::Create(b) => b.into_lua(lua),
			Self::Displace(b) => b.into_lua(lua),
			Self::DisplaceDo(b) => b.into_lua(lua),
			Self::Download(b) => b.into_lua(lua),
			Self::Enter(b) => b.into_lua(lua),
			Self::Escape(b) => b.into_lua(lua),
			Self::EscapeFilter(b) => b.into_lua(lua),
			Self::EscapeFind(b) => b.into_lua(lua),
			Self::EscapeSearch(b) => b.into_lua(lua),
			Self::EscapeSelect(b) => b.into_lua(lua),
			Self::EscapeVisual(b) => b.into_lua(lua),
			Self::Filter(b) => b.into_lua(lua),
			Self::FilterDo(b) => b.into_lua(lua),
			Self::Find(b) => b.into_lua(lua),
			Self::FindArrow(b) => b.into_lua(lua),
			Self::FindDo(b) => b.into_lua(lua),
			Self::Follow(b) => b.into_lua(lua),
			Self::Forward(b) => b.into_lua(lua),
			Self::Hardlink(b) => b.into_lua(lua),
			Self::Hidden(b) => b.into_lua(lua),
			Self::Hover(b) => b.into_lua(lua),
			Self::Leave(b) => b.into_lua(lua),
			Self::Linemode(b) => b.into_lua(lua),
			Self::Link(b) => b.into_lua(lua),
			Self::Open(b) => b.into_lua(lua),
			Self::OpenDo(b) => b.into_lua(lua),
			Self::Paste(b) => b.into_lua(lua),
			Self::Peek(b) => b.into_lua(lua),
			Self::Quit(b) => b.into_lua(lua),
			Self::Refresh(b) => b.into_lua(lua),
			Self::Remove(b) => b.into_lua(lua),
			Self::RemoveDo(b) => b.into_lua(lua),
			Self::Rename(b) => b.into_lua(lua),
			Self::Reveal(b) => b.into_lua(lua),
			Self::Search(b) => b.into_lua(lua),
			Self::SearchDo(b) => b.into_lua(lua),
			Self::SearchStop(b) => b.into_lua(lua),
			Self::Seek(b) => b.into_lua(lua),
			Self::Shell(b) => b.into_lua(lua),
			Self::Sort(b) => b.into_lua(lua),
			Self::Spot(b) => b.into_lua(lua),
			Self::Stash(b) => b.into_lua(lua),
			Self::Suspend(b) => b.into_lua(lua),
			Self::TabClose(b) => b.into_lua(lua),
			Self::TabCreate(b) => b.into_lua(lua),
			Self::TabSwap(b) => b.into_lua(lua),
			Self::TabSwitch(b) => b.into_lua(lua),
			Self::Toggle(b) => b.into_lua(lua),
			Self::ToggleAll(b) => b.into_lua(lua),
			Self::Unyank(b) => b.into_lua(lua),
			Self::UpdateFiles(b) => b.into_lua(lua),
			Self::UpdateMimes(b) => b.into_lua(lua),
			Self::UpdatePaged(b) => b.into_lua(lua),
			Self::UpdatePeeked(b) => b.into_lua(lua),
			Self::UpdateSpotted(b) => b.into_lua(lua),
			Self::UpdateYanked(b) => b.into_lua(lua),
			Self::Upload(b) => b.into_lua(lua),
			Self::VisualMode(b) => b.into_lua(lua),
			Self::Watch(b) => b.into_lua(lua),
			Self::Yank(b) => b.into_lua(lua),

			// Cmp
			Self::CmpArrow(b) => b.into_lua(lua),
			Self::CmpClose(b) => b.into_lua(lua),
			Self::CmpShow(b) => b.into_lua(lua),
			Self::CmpTrigger(b) => b.into_lua(lua),

			// Confirm
			Self::ConfirmArrow(b) => b.into_lua(lua),
			Self::ConfirmClose(b) => b.into_lua(lua),
			Self::ConfirmShow(b) => b.into_lua(lua),

			// Help
			Self::HelpArrow(b) => b.into_lua(lua),
			Self::HelpEscape(b) => b.into_lua(lua),
			Self::HelpFilter(b) => b.into_lua(lua),
			Self::HelpToggle(b) => b.into_lua(lua),

			// Input
			Self::InputBackspace(b) => b.into_lua(lua),
			Self::InputBackward(b) => b.into_lua(lua),
			Self::InputClose(b) => b.into_lua(lua),
			Self::InputComplete(b) => b.into_lua(lua),
			Self::InputDelete(b) => b.into_lua(lua),
			Self::InputEscape(b) => b.into_lua(lua),
			Self::InputForward(b) => b.into_lua(lua),
			Self::InputInsert(b) => b.into_lua(lua),
			Self::InputKill(b) => b.into_lua(lua),
			Self::InputMove(b) => b.into_lua(lua),
			Self::InputPaste(b) => b.into_lua(lua),
			Self::InputShow(b) => b.into_lua(lua),

			// Notify
			Self::NotifyTick(b) => b.into_lua(lua),

			// Pick
			Self::PickArrow(b) => b.into_lua(lua),
			Self::PickClose(b) => b.into_lua(lua),
			Self::PickShow(b) => b.into_lua(lua),

			// Spot
			Self::SpotArrow(b) => b.into_lua(lua),
			Self::SpotClose(b) => b.into_lua(lua),
			Self::SpotCopy(b) => b.into_lua(lua),
			Self::SpotSwipe(b) => b.into_lua(lua),

			// Tasks
			Self::TasksArrow(b) => b.into_lua(lua),
			Self::TasksCancel(b) => b.into_lua(lua),
			Self::TasksClose(b) => b.into_lua(lua),
			Self::TasksInspect(b) => b.into_lua(lua),
			Self::TasksOpenShellCompat(b) => b.into_lua(lua),
			Self::TasksProcessOpen(b) => b.into_lua(lua),
			Self::TasksShow(b) => b.into_lua(lua),
			Self::TasksUpdateSucceed(b) => b.into_lua(lua),

			// Which
			Self::WhichCallback(b) => b.into_lua(lua),
			Self::WhichShow(b) => b.into_lua(lua),
		}
	}
}

try_from_spark!(
	VoidOpt,
	mgr:back,
	mgr:bulk_rename,
	mgr:enter,
	mgr:escape_filter,
	mgr:escape_find,
	mgr:escape_search,
	mgr:escape_select,
	mgr:escape_visual,
	mgr:follow,
	mgr:forward,
	mgr:leave,
	mgr:refresh,
	mgr:search_stop,
	mgr:suspend,
	mgr:unyank,
	mgr:watch
);
try_from_spark!(ArrowOpt, mgr:arrow, mgr:tab_swap);
try_from_spark!(cmp::CloseOpt, cmp:close);
try_from_spark!(cmp::ShowOpt, cmp:show);
try_from_spark!(cmp::TriggerOpt, cmp:trigger);
try_from_spark!(confirm::CloseOpt, confirm:close);
try_from_spark!(confirm::ShowOpt, confirm:show);
try_from_spark!(help::ToggleOpt, help:toggle);
try_from_spark!(input::BackspaceOpt, input:backspace);
try_from_spark!(input::BackwardOpt, input:backward);
try_from_spark!(input::CloseOpt, input:close);
try_from_spark!(input::CompleteOpt, input:complete);
try_from_spark!(input::DeleteOpt, input:delete);
try_from_spark!(input::ForwardOpt, input:forward);
try_from_spark!(input::InsertOpt, input:insert);
try_from_spark!(input::KillOpt, input:kill);
try_from_spark!(input::MoveOpt, input:move);
try_from_spark!(input::PasteOpt, input:paste);
try_from_spark!(input::ShowOpt, input:show);
try_from_spark!(mgr::CdOpt, mgr:cd);
try_from_spark!(mgr::CloseOpt, mgr:close);
try_from_spark!(mgr::CopyOpt, mgr:copy);
try_from_spark!(mgr::CreateOpt, mgr:create);
try_from_spark!(mgr::DisplaceDoOpt, mgr:displace_do);
try_from_spark!(mgr::DownloadOpt, mgr:download);
try_from_spark!(mgr::EscapeOpt, mgr:escape);
try_from_spark!(mgr::FilterOpt, mgr:filter, mgr:filter_do);
try_from_spark!(mgr::FindArrowOpt, mgr:find_arrow);
try_from_spark!(mgr::FindDoOpt, mgr:find_do);
try_from_spark!(mgr::FindOpt, mgr:find);
try_from_spark!(mgr::HardlinkOpt, mgr:hardlink);
try_from_spark!(mgr::HiddenOpt, mgr:hidden);
try_from_spark!(mgr::HoverOpt, mgr:hover);
try_from_spark!(mgr::LinemodeOpt, mgr:linemode);
try_from_spark!(mgr::LinkOpt, mgr:link);
try_from_spark!(mgr::OpenDoOpt, mgr:open_do);
try_from_spark!(mgr::OpenOpt, mgr:open);
try_from_spark!(mgr::PasteOpt, mgr:paste);
try_from_spark!(mgr::PeekOpt, mgr:peek);
try_from_spark!(mgr::QuitOpt, mgr:quit);
try_from_spark!(mgr::RemoveOpt, mgr:remove, mgr:remove_do);
try_from_spark!(mgr::RenameOpt, mgr:rename);
try_from_spark!(mgr::RevealOpt, mgr:reveal);
try_from_spark!(mgr::SearchOpt, mgr:search, mgr:search_do);
try_from_spark!(mgr::SeekOpt, mgr:seek);
try_from_spark!(mgr::ShellOpt, mgr:shell);
try_from_spark!(mgr::SortOpt, mgr:sort);
try_from_spark!(mgr::SpotOpt, mgr:spot);
try_from_spark!(mgr::StashOpt, mgr:stash);
try_from_spark!(mgr::TabCloseOpt, mgr:tab_close);
try_from_spark!(mgr::TabCreateOpt, mgr:tab_create);
try_from_spark!(mgr::TabSwitchOpt, mgr:tab_switch);
try_from_spark!(mgr::ToggleAllOpt, mgr:toggle_all);
try_from_spark!(mgr::ToggleOpt, mgr:toggle);
try_from_spark!(mgr::UpdateFilesOpt, mgr:update_files);
try_from_spark!(mgr::UpdateMimesOpt, mgr:update_mimes);
try_from_spark!(mgr::UpdatePagedOpt, mgr:update_paged);
try_from_spark!(mgr::UpdatePeekedOpt, mgr:update_peeked);
try_from_spark!(mgr::UpdateSpottedOpt, mgr:update_spotted);
try_from_spark!(mgr::UpdateYankedOpt<'a>, mgr:update_yanked);
try_from_spark!(mgr::UploadOpt, mgr:upload);
try_from_spark!(mgr::VisualModeOpt, mgr:visual_mode);
try_from_spark!(mgr::YankOpt, mgr:yank);
try_from_spark!(notify::TickOpt, notify:tick);
try_from_spark!(pick::CloseOpt, pick:close);
try_from_spark!(pick::ShowOpt, pick:show);
try_from_spark!(spot::CopyOpt, spot:copy);
try_from_spark!(tasks::ProcessOpenOpt, tasks:process_open);
try_from_spark!(tasks::UpdateSucceedOpt, tasks:update_succeed);
try_from_spark!(which::CallbackOpt, which:callback);
try_from_spark!(which::ShowOpt, which:show);
