#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FolderStage {
	#[default]
	Loading,
	Loaded,
	Failed(std::io::ErrorKind, String),
}
