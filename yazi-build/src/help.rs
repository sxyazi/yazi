use yazi_macro::outln;

pub(super) struct Help;

impl Help {
	pub(super) fn run() -> anyhow::Result<()> {
		outln!(
			"Yazi build tasks\n\n\
			 Usage:\n\
			   cargo xtask build [--target <target>]\n\
			   cargo xtask dist --target <target>\n\
			   cargo xtask install [--bin-dir <path>]"
		)?;
		Ok(())
	}
}
