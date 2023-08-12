use std::path::PathBuf;

use rand::{distributions::Alphanumeric, Rng};

const FILE_PREFIX: &str = "yazi-";
const ID_LEN: usize = 10;

/// generate a path under the system temporary directory
///
/// `ext`: an ASCII string
pub fn temp_path(ext: Option<&str>) -> PathBuf {
	let (ext, suffix_len) = ext.map(|ext| (ext, ext.len())).unwrap_or_default();

	let mut name = String::with_capacity(FILE_PREFIX.len() + ID_LEN + suffix_len);
	name.push_str(FILE_PREFIX);
	rand::thread_rng().sample_iter(&Alphanumeric).take(ID_LEN).for_each(|c| name.push(c as char));

	if !ext.is_empty() {
		name.push('.');
		name.push_str(ext);
	}

	let tmp = std::env::temp_dir();
	tmp.join(&name)
}

#[cfg(test)]
mod tests {
	use super::temp_path;

	#[test]
	fn test_temp_path() {
		let p = temp_path(Some("txt"));
		println!("{p:?}");
	}
}
