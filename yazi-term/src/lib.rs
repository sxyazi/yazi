mod macros;

yazi_macro::mod_flat!(dimension error semaphore style term timeout);

yazi_macro::mod_pub!(event parser restorer sequence source stream terminal);

pub fn init() -> anyhow::Result<()> {
	YIELD_TO_SUBPROCESS.init(tokio::sync::Semaphore::new(1));

	TERM.init(terminal::Terminal::new(&yazi_tty::TTY)?);

	Ok(())
}
