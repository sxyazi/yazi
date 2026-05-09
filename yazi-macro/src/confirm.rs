#[macro_export]
macro_rules! confirm {
	($cx:ident, $cfg:expr) => {{
		let token = yazi_shared::CompletionToken::default();
		match $crate::act!(confirm:show, $cx, yazi_parser::confirm::ShowForm { cfg: $cfg, token: token.clone() }) {
			Ok(_) => Ok(token),
			Err(e) => Err(e)
		}
	}};
}
