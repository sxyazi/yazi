#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathDyn<'p> {
	Os(&'p std::path::Path),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathBufDyn {
	Os(std::path::PathBuf),
}
