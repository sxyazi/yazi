use strum::{Display, IntoStaticStr};

#[derive(Clone, Copy, Debug, Display, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum SparkKind {
	// app:title
	IndAppTitle,

	// mgr:close
	KeyClose,
	// mgr:hidden
	KeyHidden,
	IndHidden,
	// mgr:sort
	KeySort,
	IndSort,
	// mgr:stash
	IndStash,
	RelayStash,
	// mgr:quit
	KeyQuit,

	// which:activate
	IndWhichActivate,

	// notify:push
	RelayNotifyPush,
}
