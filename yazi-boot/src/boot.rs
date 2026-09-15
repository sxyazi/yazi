use yazi_fs::CWD;
use yazi_shared::url::UrlBuf;

#[derive(Debug)]
pub struct Boot {
	pub cwd: UrlBuf,
}

impl Default for Boot {
	fn default() -> Self { Self { cwd: CWD.load().as_ref().clone() } }
}
