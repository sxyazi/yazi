use mlua::{FromLua, IntoLua, Lua, Value};

use crate::{spark::SparkKind, try_from_spark};

#[derive(Debug)]
pub enum Spark<'a> {
	// Void
	Void(crate::VoidOpt),

	// App
	AppAcceptPayload(yazi_dds::Payload<'a>),
	AppBootstrap(crate::VoidOpt),
	AppDeprecate(crate::app::DeprecateOpt),
	AppFocus(crate::VoidOpt),
	AppLua(crate::app::LuaOpt),
	AppMouse(crate::app::MouseOpt),
	AppPlugin(crate::app::PluginForm),
	AppPluginDo(crate::app::PluginForm),
	AppQuit(crate::app::QuitForm),
	AppReflow(crate::app::ReflowOpt),
	AppResize(crate::app::ReflowOpt),
	AppResume(crate::app::ResumeOpt),
	AppStop(crate::app::StopOpt),
	AppTitle(crate::app::TitleOpt),
	AppUpdateProgress(crate::app::UpdateProgressOpt),

	// Mgr
	Arrow(crate::ArrowOpt),
	Back(crate::VoidOpt),
	BulkExit(crate::mgr::BulkExitOpt),
	BulkRename(crate::VoidOpt),
	Cd(crate::mgr::CdOpt),
	Close(crate::mgr::CloseForm),
	Copy(crate::mgr::CopyOpt),
	Create(crate::mgr::CreateOpt),
	Displace(crate::VoidOpt),
	DisplaceDo(crate::mgr::DisplaceDoForm),
	Download(crate::mgr::DownloadOpt),
	Enter(crate::VoidOpt),
	Escape(crate::mgr::EscapeOpt),
	EscapeFilter(crate::VoidOpt),
	EscapeFind(crate::VoidOpt),
	EscapeSearch(crate::VoidOpt),
	EscapeSelect(crate::VoidOpt),
	EscapeVisual(crate::VoidOpt),
	Filter(crate::mgr::FilterForm),
	FilterDo(crate::mgr::FilterForm),
	Find(crate::mgr::FindOpt),
	FindArrow(crate::mgr::FindArrowOpt),
	FindDo(crate::mgr::FindDoForm),
	Follow(crate::VoidOpt),
	Forward(crate::VoidOpt),
	Hardlink(crate::mgr::HardlinkOpt),
	Hidden(crate::mgr::HiddenOpt),
	Hover(crate::mgr::HoverOpt),
	Leave(crate::VoidOpt),
	Linemode(crate::mgr::LinemodeOpt),
	Link(crate::mgr::LinkOpt),
	Open(crate::mgr::OpenForm),
	OpenDo(crate::mgr::OpenDoForm),
	Paste(crate::mgr::PasteOpt),
	Peek(crate::mgr::PeekOpt),
	Quit(crate::app::QuitForm),
	Refresh(crate::VoidOpt),
	Remove(crate::mgr::RemoveOpt),
	RemoveDo(crate::mgr::RemoveOpt),
	Rename(crate::mgr::RenameOpt),
	Reveal(crate::mgr::RevealOpt),
	Search(crate::mgr::SearchForm),
	SearchDo(crate::mgr::SearchForm),
	SearchStop(crate::VoidOpt),
	Seek(crate::mgr::SeekOpt),
	Shell(crate::mgr::ShellOpt),
	Sort(crate::mgr::SortOpt),
	Spot(crate::mgr::SpotOpt),
	Stash(crate::mgr::StashOpt),
	Suspend(crate::VoidOpt),
	TabClose(crate::mgr::TabCloseOpt),
	TabCreate(crate::mgr::TabCreateOpt),
	TabRename(crate::mgr::TabRenameOpt),
	TabSwap(crate::ArrowOpt),
	TabSwitch(crate::mgr::TabSwitchOpt),
	Toggle(crate::mgr::ToggleOpt),
	ToggleAll(crate::mgr::ToggleAllOpt),
	Unyank(crate::VoidOpt),
	UpdateFiles(crate::mgr::UpdateFilesOpt),
	UpdateMimes(crate::mgr::UpdateMimesOpt),
	UpdatePaged(crate::mgr::UpdatePagedOpt),
	UpdatePeeked(crate::mgr::UpdatePeekedForm),
	UpdateSpotted(crate::mgr::UpdateSpottedForm),
	UpdateYanked(crate::mgr::UpdateYankedOpt<'a>),
	Upload(crate::mgr::UploadOpt),
	VisualMode(crate::mgr::VisualModeOpt),
	Watch(crate::VoidOpt),
	Yank(crate::mgr::YankOpt),

	// Cmp
	CmpArrow(crate::ArrowOpt),
	CmpClose(crate::cmp::CloseOpt),
	CmpShow(crate::cmp::ShowForm),
	CmpTrigger(crate::cmp::TriggerOpt),

	// Confirm
	ConfirmArrow(crate::ArrowOpt),
	ConfirmClose(crate::confirm::CloseOpt),
	ConfirmShow(Box<crate::confirm::ShowOpt>),

	// Help
	HelpArrow(crate::ArrowOpt),
	HelpEscape(crate::VoidOpt),
	HelpFilter(crate::VoidOpt),
	HelpToggle(crate::help::ToggleOpt),

	// Input
	InputBackspace(yazi_widgets::input::parser::BackspaceOpt),
	InputBackward(yazi_widgets::input::parser::BackwardOpt),
	InputClose(crate::input::CloseOpt),
	InputComplete(yazi_widgets::input::parser::CompleteOpt),
	InputDelete(yazi_widgets::input::parser::DeleteOpt),
	InputEscape(crate::VoidOpt),
	InputForward(yazi_widgets::input::parser::ForwardOpt),
	InputInsert(yazi_widgets::input::parser::InsertOpt),
	InputKill(yazi_widgets::input::parser::KillOpt),
	InputMove(yazi_widgets::input::parser::MoveOpt),
	InputPaste(yazi_widgets::input::parser::PasteOpt),
	InputShow(yazi_widgets::input::InputOpt),

	// Notify
	NotifyPush(crate::notify::PushForm),
	NotifyTick(crate::notify::TickOpt),

	// Pick
	PickArrow(crate::ArrowOpt),
	PickClose(crate::pick::CloseOpt),
	PickShow(crate::pick::ShowOpt),

	// Spot
	SpotArrow(crate::ArrowOpt),
	SpotClose(crate::VoidOpt),
	SpotCopy(crate::spot::CopyOpt),
	SpotSwipe(crate::ArrowOpt),

	// Tasks
	TasksArrow(crate::ArrowOpt),
	TasksCancel(crate::VoidOpt),
	TasksClose(crate::VoidOpt),
	TasksInspect(crate::VoidOpt),
	TasksOpenShellCompat(crate::tasks::ProcessOpenForm),
	TasksProcessOpen(crate::tasks::ProcessOpenForm),
	TasksShow(crate::VoidOpt),
	TasksUpdateSucceed(crate::tasks::UpdateSucceedOpt),

	// Which
	WhichActivate(crate::which::ActivateForm),
	WhichDismiss(crate::VoidOpt),
}

impl<'a> Spark<'a> {
	pub fn from_lua(lua: &Lua, kind: SparkKind, value: Value) -> mlua::Result<Self> {
		use SparkKind::*;

		Ok(match kind {
			// app:title
			IndAppTitle => Self::AppTitle(<_>::from_lua(value, lua)?),

			// mgr:hidden
			KeyHidden => Self::Hidden(<_>::from_lua(value, lua)?),
			IndHidden => Self::Hidden(<_>::from_lua(value, lua)?),
			// mgr:sort
			KeySort => Self::Sort(<_>::from_lua(value, lua)?),
			IndSort => Self::Sort(<_>::from_lua(value, lua)?),
			// mgr:stash
			IndStash => Self::Stash(<_>::from_lua(value, lua)?),
			RelayStash => Self::Stash(<_>::from_lua(value, lua)?),
			// mgr:quit
			KeyQuit => Self::Quit(<_>::from_lua(value, lua)?),

			// which:activate
			IndWhichActivate => Self::WhichActivate(<_>::from_lua(value, lua)?),

			// notify:push
			RelayNotifyPush => Self::NotifyPush(<_>::from_lua(value, lua)?),
		})
	}
}

impl<'a> IntoLua for Spark<'a> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			// Void
			Self::Void(b) => b.into_lua(lua),

			// App
			Self::AppAcceptPayload(b) => b.into_lua(lua),
			Self::AppBootstrap(b) => b.into_lua(lua),
			Self::AppDeprecate(b) => b.into_lua(lua),
			Self::AppFocus(b) => b.into_lua(lua),
			Self::AppLua(b) => b.into_lua(lua),
			Self::AppMouse(b) => b.into_lua(lua),
			Self::AppPlugin(b) => b.into_lua(lua),
			Self::AppPluginDo(b) => b.into_lua(lua),
			Self::AppQuit(b) => b.into_lua(lua),
			Self::AppReflow(b) => b.into_lua(lua),
			Self::AppResize(b) => b.into_lua(lua),
			Self::AppResume(b) => b.into_lua(lua),
			Self::AppStop(b) => b.into_lua(lua),
			Self::AppTitle(b) => b.into_lua(lua),
			Self::AppUpdateProgress(b) => b.into_lua(lua),

			// Mgr
			Self::Arrow(b) => b.into_lua(lua),
			Self::Back(b) => b.into_lua(lua),
			Self::BulkExit(b) => b.into_lua(lua),
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
			Self::TabRename(b) => b.into_lua(lua),
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
			Self::NotifyPush(b) => b.into_lua(lua),
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
			Self::WhichActivate(b) => b.into_lua(lua),
			Self::WhichDismiss(b) => b.into_lua(lua),
		}
	}
}

try_from_spark!(
	crate::VoidOpt,
	app:bootstrap,
	app:focus,
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
	mgr:watch,
	which:dismiss
);

// App
try_from_spark!(crate::ArrowOpt, mgr:arrow, mgr:tab_swap);
try_from_spark!(crate::app::DeprecateOpt, app:deprecate);
try_from_spark!(crate::app::LuaOpt, app:lua);
try_from_spark!(crate::app::MouseOpt, app:mouse);
try_from_spark!(crate::app::PluginForm, app:plugin, app:plugin_do);
try_from_spark!(crate::app::QuitForm, app:quit, mgr:quit);
try_from_spark!(crate::app::ReflowOpt, app:reflow, app:resize);
try_from_spark!(crate::app::ResumeOpt, app:resume);
try_from_spark!(crate::app::StopOpt, app:stop);
try_from_spark!(crate::app::TitleOpt, app:title);
try_from_spark!(crate::app::UpdateProgressOpt, app:update_progress);
try_from_spark!(crate::cmp::CloseOpt, cmp:close);
try_from_spark!(crate::cmp::ShowForm, cmp:show);
try_from_spark!(crate::cmp::TriggerOpt, cmp:trigger);
try_from_spark!(crate::confirm::CloseOpt, confirm:close);
try_from_spark!(crate::confirm::ShowOpt, confirm:show);
try_from_spark!(crate::help::ToggleOpt, help:toggle);
try_from_spark!(crate::input::CloseOpt, input:close);
try_from_spark!(crate::mgr::BulkExitOpt, mgr:bulk_exit);
try_from_spark!(crate::mgr::CdOpt, mgr:cd);
try_from_spark!(crate::mgr::CloseForm, mgr:close);
try_from_spark!(crate::mgr::CopyOpt, mgr:copy);
try_from_spark!(crate::mgr::CreateOpt, mgr:create);
try_from_spark!(crate::mgr::DisplaceDoForm, mgr:displace_do);
try_from_spark!(crate::mgr::DownloadOpt, mgr:download);
try_from_spark!(crate::mgr::EscapeOpt, mgr:escape);
try_from_spark!(crate::mgr::FilterForm, mgr:filter, mgr:filter_do);
try_from_spark!(crate::mgr::FindArrowOpt, mgr:find_arrow);
try_from_spark!(crate::mgr::FindDoForm, mgr:find_do);
try_from_spark!(crate::mgr::FindOpt, mgr:find);
try_from_spark!(crate::mgr::HardlinkOpt, mgr:hardlink);
try_from_spark!(crate::mgr::HiddenOpt, mgr:hidden);
try_from_spark!(crate::mgr::HoverOpt, mgr:hover);
try_from_spark!(crate::mgr::LinemodeOpt, mgr:linemode);
try_from_spark!(crate::mgr::LinkOpt, mgr:link);
try_from_spark!(crate::mgr::OpenDoForm, mgr:open_do);
try_from_spark!(crate::mgr::OpenForm, mgr:open);
try_from_spark!(crate::mgr::PasteOpt, mgr:paste);
try_from_spark!(crate::mgr::PeekOpt, mgr:peek);
try_from_spark!(crate::mgr::RemoveOpt, mgr:remove, mgr:remove_do);
try_from_spark!(crate::mgr::RenameOpt, mgr:rename);
try_from_spark!(crate::mgr::RevealOpt, mgr:reveal);
try_from_spark!(crate::mgr::SearchForm, mgr:search, mgr:search_do);
try_from_spark!(crate::mgr::SeekOpt, mgr:seek);
try_from_spark!(crate::mgr::ShellOpt, mgr:shell);
try_from_spark!(crate::mgr::SortOpt, mgr:sort);
try_from_spark!(crate::mgr::SpotOpt, mgr:spot);
try_from_spark!(crate::mgr::StashOpt, mgr:stash);
try_from_spark!(crate::mgr::TabCloseOpt, mgr:tab_close);
try_from_spark!(crate::mgr::TabCreateOpt, mgr:tab_create);
try_from_spark!(crate::mgr::TabRenameOpt, mgr:tab_rename);
try_from_spark!(crate::mgr::TabSwitchOpt, mgr:tab_switch);
try_from_spark!(crate::mgr::ToggleAllOpt, mgr:toggle_all);
try_from_spark!(crate::mgr::ToggleOpt, mgr:toggle);
try_from_spark!(crate::mgr::UpdateFilesOpt, mgr:update_files);
try_from_spark!(crate::mgr::UpdateMimesOpt, mgr:update_mimes);
try_from_spark!(crate::mgr::UpdatePagedOpt, mgr:update_paged);
try_from_spark!(crate::mgr::UpdatePeekedForm, mgr:update_peeked);
try_from_spark!(crate::mgr::UpdateSpottedForm, mgr:update_spotted);
try_from_spark!(crate::mgr::UpdateYankedOpt<'a>, mgr:update_yanked);
try_from_spark!(crate::mgr::UploadOpt, mgr:upload);
try_from_spark!(crate::mgr::VisualModeOpt, mgr:visual_mode);
try_from_spark!(crate::mgr::YankOpt, mgr:yank);
try_from_spark!(crate::notify::PushForm, notify:push);
try_from_spark!(crate::notify::TickOpt, notify:tick);
try_from_spark!(crate::pick::CloseOpt, pick:close);
try_from_spark!(crate::pick::ShowOpt, pick:show);
try_from_spark!(crate::spot::CopyOpt, spot:copy);
try_from_spark!(crate::tasks::ProcessOpenForm, tasks:process_open);
try_from_spark!(crate::tasks::UpdateSucceedOpt, tasks:update_succeed);
try_from_spark!(crate::which::ActivateForm, which:activate);
try_from_spark!(yazi_dds::Payload<'a>, app:accept_payload);
try_from_spark!(yazi_widgets::input::InputOpt, input:show);
try_from_spark!(yazi_widgets::input::parser::BackspaceOpt, input:backspace);
try_from_spark!(yazi_widgets::input::parser::BackwardOpt, input:backward);
try_from_spark!(yazi_widgets::input::parser::CompleteOpt, input:complete);
try_from_spark!(yazi_widgets::input::parser::DeleteOpt, input:delete);
try_from_spark!(yazi_widgets::input::parser::ForwardOpt, input:forward);
try_from_spark!(yazi_widgets::input::parser::InsertOpt, input:insert);
try_from_spark!(yazi_widgets::input::parser::KillOpt, input:kill);
try_from_spark!(yazi_widgets::input::parser::MoveOpt, input:move);
try_from_spark!(yazi_widgets::input::parser::PasteOpt, input:paste);
