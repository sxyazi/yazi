#[macro_export]
macro_rules! ctx {
	($task:ident, $result:expr) => {{
		use anyhow::Context;
		$result.with_context(|| format!("Failed to work on {:?}", $task))
	}};
	($task:ident, $result:expr, $($args:tt)*) => {{
		use anyhow::Context;
		$result.with_context(|| format!("Failed to work on {:?}: {}", $task, format_args!($($args)*)))
	}};
}

#[macro_export]
macro_rules! ok_or_not_found {
	($task:ident, $result:expr, $not_found:expr) => {
		match $result {
			Ok(v) => v,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => $not_found,
			Err(e) => $crate::ctx!($task, Err(e))?,
		}
	};
	($task:ident, $result:expr) => {
		ok_or_not_found!($task, $result, Default::default())
	};
}

#[macro_export]
macro_rules! progress_or_break {
	($rx:ident, $done:expr) => {
		tokio::select! {
			r = $rx.recv() => {
				match r {
					Some(prog) => prog,
					None => break,
				}
			},
			false = $done.future() => break,
		}
	};
}

#[macro_export]
macro_rules! impl_from_in {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskIn {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}

#[macro_export]
macro_rules! impl_from_out {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskOut {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}

#[macro_export]
macro_rules! impl_from_prog {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskProg {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}
