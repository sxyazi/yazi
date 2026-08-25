use strum::{EnumIs, FromRepr};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, FromRepr, PartialEq)]
pub enum TaskStatus {
	#[default]
	Pending   = 0,
	Started   = 1,
	Succeeded = 2,
	Failed    = 3,
	Canceled  = 4,
}

impl TaskStatus {
	pub(crate) fn has_started(self) -> bool {
		matches!(self, Self::Started | Self::Succeeded | Self::Failed)
	}

	pub(crate) fn is_cancelable(self) -> bool {
		matches!(self, Self::Pending | Self::Started | Self::Failed)
	}

	pub(crate) fn is_finishable(self) -> bool { matches!(self, Self::Pending | Self::Started) }

	pub(crate) fn is_finished(self) -> bool {
		matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
	}
}
