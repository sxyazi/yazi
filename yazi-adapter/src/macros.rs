#[macro_export]
macro_rules! tmux {
	($s:literal) => {
		if *$crate::TMUX {
			std::borrow::Cow::Owned(format!(
				"{}{}{}",
				*$crate::START,
				$s.trim_start_matches('\x1b').replace('\x1b', *$crate::ESCAPE),
				*$crate::CLOSE
			))
		} else {
			std::borrow::Cow::Borrowed($s)
		}
	};
}
