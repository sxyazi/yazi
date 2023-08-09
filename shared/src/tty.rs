use crossterm::terminal::window_size;

#[inline]
pub fn tty_ratio() -> (f64, f64) {
	if let Ok(ws) = window_size() {
		(f64::from(ws.width) / f64::from(ws.columns), f64::from(ws.height) / f64::from(ws.rows))
	} else {
		(1f64, 1f64)
	}
}
