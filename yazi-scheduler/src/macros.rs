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

#[macro_export]
macro_rules! dispatch_progress {
	($value:expr, $method:ident) => {
		match $value {
			// File
			$crate::TaskProg::FileCopy(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileCut(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileLink(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileHardlink(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileDelete(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileTrash(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileDownload(p) => $crate::Progress::$method(p),
			$crate::TaskProg::FileUpload(p) => $crate::Progress::$method(p),
			// Plugin
			$crate::TaskProg::PluginEntry(p) => $crate::Progress::$method(p),
			// Prework
			$crate::TaskProg::Fetch(p) => $crate::Progress::$method(p),
			$crate::TaskProg::Preload(p) => $crate::Progress::$method(p),
			$crate::TaskProg::Size(p) => $crate::Progress::$method(p),
			// Process
			$crate::TaskProg::ProcessBlock(p) => $crate::Progress::$method(p),
			$crate::TaskProg::ProcessOrphan(p) => $crate::Progress::$method(p),
			$crate::TaskProg::ProcessBg(p) => $crate::Progress::$method(p),
		}
	};
}
