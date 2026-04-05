use serde::Serialize;
use strum::EnumIs;

#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CleanupState {
	#[default]
	Pending,
	Success,
	Failed,
}
