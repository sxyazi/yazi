mod macros;

yazi_macro::mod_flat!(dimension error semaphore term timeout);

yazi_macro::mod_pub!(event parser restorer source stream terminal waker);

pub fn init() -> anyhow::Result<()> {
	YIELD_TO_SUBPROCESS.init(tokio::sync::Semaphore::new(1));

	TERM.init(terminal::Terminal::new(&yazi_tty::TTY)?);

	Ok(())
}
