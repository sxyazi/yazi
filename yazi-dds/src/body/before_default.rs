use anyhow::bail;

use crate::body::Body;

macro_rules! unsupported {
	[ $( $layer:ident : $name:ident $(<$lt:lifetime>)? $(,)? )+ ] => {
		$(
			paste::paste! {
				unsupported!(yazi_parser::$layer::[<$name:camel Opt>] $(<$lt>)?);
			}
		)+
	};
	[ $( $path:path $(,)? )+ ] => {
		$(
			impl TryFrom<&$path> for Body<'static> {
				type Error = anyhow::Error;

				fn try_from(_: &$path) -> Result<Self, Self::Error> {
					bail!("unsupported");
				}
			}
		)+
	};
}

unsupported!(
	mgr:cd
	mgr:close
	mgr:copy
	mgr:create
	mgr:escape
	mgr:filter
	mgr:find
	mgr:find_arrow
	mgr:find_do
	mgr:forward
	mgr:hardlink
	mgr:hidden
	mgr:linemode
	mgr:link
	mgr:open
	mgr:open_do
	mgr:paste
	mgr:peek
	mgr:remove
	mgr:rename
	mgr:reveal
	mgr:search
	mgr:seek
	mgr:shell
	mgr:spot
	mgr:tab_close
	mgr:tab_create
	mgr:tab_switch
	mgr:toggle
	mgr:toggle_all
	mgr:update_files
	mgr:update_mimes
	mgr:update_paged
	mgr:update_peeked
	mgr:update_spotted
	mgr:update_tasks
	mgr:update_yanked<'_>
	mgr:visual_mode
	mgr:yank
);

unsupported!(
	yazi_parser::ArrowOpt
	yazi_parser::VoidOpt
	yazi_shared::event::CmdCow
);
