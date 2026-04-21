use mlua::{FromLua, IntoLua, Lua, Value};

use crate::{spark::SparkKind, try_from_spark};

#[derive(Debug)]
pub enum Spark<'a> {
	// Void
	Void(crate::VoidForm),

	// App
	AppAcceptPayload(yazi_dds::Payload<'a>),
	AppBootstrap(crate::VoidForm),
	AppDeprecate(crate::app::DeprecateForm),
	AppFocus(crate::VoidForm),
	AppLua(crate::app::LuaForm),
	AppMouse(crate::app::MouseForm),
	AppPlugin(crate::app::PluginForm),
	AppPluginDo(crate::app::PluginForm),
	AppQuit(crate::app::QuitForm),
	AppReflow(crate::app::ReflowForm),
	AppResize(crate::app::ReflowForm),
	AppResume(crate::app::ResumeForm),
	AppStop(crate::app::StopForm),
	AppTheme(crate::VoidForm),
	AppTitle(crate::app::TitleForm),
	AppUpdateProgress(crate::app::UpdateProgressForm),

	// Mgr
	Arrow(crate::ArrowForm),
	Back(crate::VoidForm),
	BulkExit(crate::mgr::BulkExitForm),
	BulkRename(crate::VoidForm),
	Cd(crate::mgr::CdForm),
	Close(crate::mgr::CloseForm),
	Copy(crate::mgr::CopyForm),
	Create(crate::mgr::CreateForm),
	Displace(crate::VoidForm),
	DisplaceDo(crate::mgr::DisplaceDoForm),
	Download(crate::mgr::DownloadForm),
	Enter(crate::VoidForm),
	Escape(crate::mgr::EscapeForm),
	EscapeFilter(crate::VoidForm),
	EscapeFind(crate::VoidForm),
	EscapeSearch(crate::VoidForm),
	EscapeSelect(crate::VoidForm),
	EscapeVisual(crate::VoidForm),
	Filter(crate::mgr::FilterForm),
	FilterDo(crate::mgr::FilterForm),
	Find(crate::mgr::FindForm),
	FindArrow(crate::mgr::FindArrowForm),
	FindDo(crate::mgr::FindDoForm),
	Follow(crate::VoidForm),
	Forward(crate::VoidForm),
	Hardlink(crate::mgr::HardlinkForm),
	Hidden(crate::mgr::HiddenForm),
	Hover(crate::mgr::HoverForm),
	Leave(crate::VoidForm),
	Linemode(crate::mgr::LinemodeForm),
	Link(crate::mgr::LinkForm),
	Open(crate::mgr::OpenForm),
	OpenDo(crate::mgr::OpenDoForm),
	Paste(crate::mgr::PasteForm),
	Peek(crate::mgr::PeekForm),
	Quit(crate::app::QuitForm),
	Refresh(crate::VoidForm),
	Remove(crate::mgr::RemoveForm),
	RemoveDo(crate::mgr::RemoveForm),
	Rename(crate::mgr::RenameForm),
	Reveal(crate::mgr::RevealForm),
	Search(crate::mgr::SearchForm),
	SearchDo(crate::mgr::SearchForm),
	SearchStop(crate::VoidForm),
	Seek(crate::mgr::SeekForm),
	Shell(crate::mgr::ShellForm),
	Sort(crate::mgr::SortForm),
	Spot(crate::mgr::SpotOpt),
	Stash(crate::mgr::StashForm),
	Suspend(crate::VoidForm),
	TabClose(crate::mgr::TabCloseForm),
	TabCreate(crate::mgr::TabCreateForm),
	TabRename(crate::mgr::TabRenameForm),
	TabSwap(crate::ArrowForm),
	TabSwitch(crate::mgr::TabSwitchForm),
	Toggle(crate::mgr::ToggleForm),
	ToggleAll(crate::mgr::ToggleAllForm),
	Unyank(crate::VoidForm),
	UpdateFiles(crate::mgr::UpdateFilesForm),
	UpdateMimes(crate::mgr::UpdateMimesForm),
	UpdatePaged(crate::mgr::UpdatePagedForm),
	UpdatePeeked(crate::mgr::UpdatePeekedForm),
	UpdateSpotted(crate::mgr::UpdateSpottedForm),
	UpdateYanked(crate::mgr::UpdateYankedForm<'a>),
	Upload(crate::mgr::UploadForm),
	VisualMode(crate::mgr::VisualModeForm),
	Watch(crate::VoidForm),
	Yank(crate::mgr::YankForm),

	// Cmp
	CmpArrow(crate::ArrowForm),
	CmpClose(crate::cmp::CloseForm),
	CmpShow(crate::cmp::ShowForm),
	CmpTrigger(crate::cmp::TriggerForm),

	// Confirm
	ConfirmArrow(crate::ArrowForm),
	ConfirmClose(crate::confirm::CloseForm),
	ConfirmShow(Box<crate::confirm::ShowForm>),

	// Help
	HelpArrow(crate::ArrowForm),
	HelpEscape(crate::VoidForm),
	HelpFilter(crate::VoidForm),
	HelpToggle(crate::help::ToggleForm),

	// Input
	InputBackspace(yazi_widgets::input::parser::BackspaceOpt),
	InputBackward(yazi_widgets::input::parser::BackwardOpt),
	InputClose(crate::input::CloseForm),
	InputComplete(yazi_widgets::input::parser::CompleteOpt),
	InputDelete(yazi_widgets::input::parser::DeleteOpt),
	InputEscape(crate::VoidForm),
	InputForward(yazi_widgets::input::parser::ForwardOpt),
	InputInsert(yazi_widgets::input::parser::InsertOpt),
	InputKill(yazi_widgets::input::parser::KillOpt),
	InputMove(yazi_widgets::input::parser::MoveOpt),
	InputPaste(yazi_widgets::input::parser::PasteOpt),
	InputShow(yazi_widgets::input::InputOpt),

	// Notify
	NotifyPush(crate::notify::PushForm),
	NotifyTick(crate::notify::TickForm),

	// Pick
	PickArrow(crate::ArrowForm),
	PickClose(crate::pick::CloseForm),
	PickShow(crate::pick::ShowForm),

	// Spot
	SpotArrow(crate::ArrowForm),
	SpotClose(crate::VoidForm),
	SpotCopy(crate::spot::CopyForm),
	SpotSwipe(crate::ArrowForm),

	// Tasks
	TasksArrow(crate::ArrowForm),
	TasksCancel(crate::VoidForm),
	TasksClose(crate::VoidForm),
	TasksInspect(crate::VoidForm),
	TasksOpenShellCompat(crate::tasks::ProcessOpenForm),
	TasksProcessOpen(crate::tasks::ProcessOpenForm),
	TasksShow(crate::VoidForm),
	TasksSpawn(crate::tasks::SpawnForm),
	TasksUpdateSucceed(crate::tasks::UpdateSucceedForm),

	// Which
	WhichActivate(crate::which::ActivateForm),
	WhichDismiss(crate::VoidForm),
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
			Self::AppTheme(b) => b.into_lua(lua),
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
			Self::TasksSpawn(b) => b.into_lua(lua),
			Self::TasksUpdateSucceed(b) => b.into_lua(lua),

			// Which
			Self::WhichActivate(b) => b.into_lua(lua),
			Self::WhichDismiss(b) => b.into_lua(lua),
		}
	}
}

try_from_spark!(
	crate::VoidForm,
	app:bootstrap,
	app:focus,
	app:theme,
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
try_from_spark!(crate::ArrowForm, mgr:arrow, mgr:tab_swap);
try_from_spark!(crate::app::DeprecateForm, app:deprecate);
try_from_spark!(crate::app::LuaForm, app:lua);
try_from_spark!(crate::app::MouseForm, app:mouse);
try_from_spark!(crate::app::PluginForm, app:plugin, app:plugin_do);
try_from_spark!(crate::app::QuitForm, app:quit, mgr:quit);
try_from_spark!(crate::app::ReflowForm, app:reflow, app:resize);
try_from_spark!(crate::app::ResumeForm, app:resume);
try_from_spark!(crate::app::StopForm, app:stop);
try_from_spark!(crate::app::TitleForm, app:title);
try_from_spark!(crate::app::UpdateProgressForm, app:update_progress);
try_from_spark!(crate::cmp::CloseForm, cmp:close);
try_from_spark!(crate::cmp::ShowForm, cmp:show);
try_from_spark!(crate::cmp::TriggerForm, cmp:trigger);
try_from_spark!(crate::confirm::CloseForm, confirm:close);
try_from_spark!(crate::confirm::ShowForm, confirm:show);
try_from_spark!(crate::help::ToggleForm, help:toggle);
try_from_spark!(crate::input::CloseForm, input:close);
try_from_spark!(crate::mgr::BulkExitForm, mgr:bulk_exit);
try_from_spark!(crate::mgr::CdForm, mgr:cd);
try_from_spark!(crate::mgr::CloseForm, mgr:close);
try_from_spark!(crate::mgr::CopyForm, mgr:copy);
try_from_spark!(crate::mgr::CreateForm, mgr:create);
try_from_spark!(crate::mgr::DisplaceDoForm, mgr:displace_do);
try_from_spark!(crate::mgr::DownloadForm, mgr:download);
try_from_spark!(crate::mgr::EscapeForm, mgr:escape);
try_from_spark!(crate::mgr::FilterForm, mgr:filter, mgr:filter_do);
try_from_spark!(crate::mgr::FindArrowForm, mgr:find_arrow);
try_from_spark!(crate::mgr::FindDoForm, mgr:find_do);
try_from_spark!(crate::mgr::FindForm, mgr:find);
try_from_spark!(crate::mgr::HardlinkForm, mgr:hardlink);
try_from_spark!(crate::mgr::HiddenForm, mgr:hidden);
try_from_spark!(crate::mgr::HoverForm, mgr:hover);
try_from_spark!(crate::mgr::LinemodeForm, mgr:linemode);
try_from_spark!(crate::mgr::LinkForm, mgr:link);
try_from_spark!(crate::mgr::OpenDoForm, mgr:open_do);
try_from_spark!(crate::mgr::OpenForm, mgr:open);
try_from_spark!(crate::mgr::PasteForm, mgr:paste);
try_from_spark!(crate::mgr::PeekForm, mgr:peek);
try_from_spark!(crate::mgr::RemoveForm, mgr:remove, mgr:remove_do);
try_from_spark!(crate::mgr::RenameForm, mgr:rename);
try_from_spark!(crate::mgr::RevealForm, mgr:reveal);
try_from_spark!(crate::mgr::SearchForm, mgr:search, mgr:search_do);
try_from_spark!(crate::mgr::SeekForm, mgr:seek);
try_from_spark!(crate::mgr::ShellForm, mgr:shell);
try_from_spark!(crate::mgr::SortForm, mgr:sort);
try_from_spark!(crate::mgr::SpotOpt, mgr:spot);
try_from_spark!(crate::mgr::StashForm, mgr:stash);
try_from_spark!(crate::mgr::TabCloseForm, mgr:tab_close);
try_from_spark!(crate::mgr::TabCreateForm, mgr:tab_create);
try_from_spark!(crate::mgr::TabRenameForm, mgr:tab_rename);
try_from_spark!(crate::mgr::TabSwitchForm, mgr:tab_switch);
try_from_spark!(crate::mgr::ToggleAllForm, mgr:toggle_all);
try_from_spark!(crate::mgr::ToggleForm, mgr:toggle);
try_from_spark!(crate::mgr::UpdateFilesForm, mgr:update_files);
try_from_spark!(crate::mgr::UpdateMimesForm, mgr:update_mimes);
try_from_spark!(crate::mgr::UpdatePagedForm, mgr:update_paged);
try_from_spark!(crate::mgr::UpdatePeekedForm, mgr:update_peeked);
try_from_spark!(crate::mgr::UpdateSpottedForm, mgr:update_spotted);
try_from_spark!(crate::mgr::UpdateYankedForm<'a>, mgr:update_yanked);
try_from_spark!(crate::mgr::UploadForm, mgr:upload);
try_from_spark!(crate::mgr::VisualModeForm, mgr:visual_mode);
try_from_spark!(crate::mgr::YankForm, mgr:yank);
try_from_spark!(crate::notify::PushForm, notify:push);
try_from_spark!(crate::notify::TickForm, notify:tick);
try_from_spark!(crate::pick::CloseForm, pick:close);
try_from_spark!(crate::pick::ShowForm, pick:show);
try_from_spark!(crate::spot::CopyForm, spot:copy);
try_from_spark!(crate::tasks::ProcessOpenForm, tasks:process_open);
try_from_spark!(crate::tasks::SpawnForm, tasks:spawn);
try_from_spark!(crate::tasks::UpdateSucceedForm, tasks:update_succeed);
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
