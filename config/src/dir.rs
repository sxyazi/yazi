use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub(crate) fn get_config_file(path: &str) -> PathBuf { dirs::config_dir().unwrap().join(path) }

#[cfg(not(target_os = "windows"))]
pub(crate) fn get_config_file(path: &str) -> PathBuf {
	xdg::BaseDirectories::new().unwrap().get_config_file(path)
}

#[cfg(target_os = "windows")]
pub(crate) fn get_state_dir() -> Result<PathBuf, String> {
	dirs::data_dir()
		.map(|dir| dir.join("yazi").join("state"))
		.ok_or_else(|| String::from("failed to get state directory"))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn get_state_dir() -> Result<PathBuf, String> {
	xdg::BaseDirectories::with_prefix("yazi")
		.map(|dirs| dirs.get_state_home())
		.map_err(|e| e.to_string())
}
