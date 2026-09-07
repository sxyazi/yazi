macro_rules! physical {
	($p:ident, $method:ident $(, $arg:expr)*) => {
		match $crate::engine::Engines::new($p.url.physical()).await? {
			$crate::engine::Engines::Local(p) => p.$method($($arg),*).await.map(Into::into),
			$crate::engine::Engines::Lua(p) => p.$method($($arg),*).await.map(Into::into),
			$crate::engine::Engines::Sftp(p) => p.$method($($arg),*).await.map(Into::into),
		}
	};
}

macro_rules! dispatch {
	($me:expr, $method:ident $(, $arg:expr)*) => {{
		use yazi_macro::capability;

		match $me {
			$crate::engine::Engines::Local(p) => p.$method($($arg),*).await,
			$crate::engine::Engines::Lua(p) if p.handles(capability!($method)).await? => p.$method($($arg),*).await,
			$crate::engine::Engines::Lua(p) => physical!(p, $method $(, $arg)*),
			$crate::engine::Engines::Sftp(p) => p.$method($($arg),*).await,
		}}
	};
}

macro_rules! poll_rw {
	($me:expr, $method:ident $(, $arg:expr)*) => {{
		use std::pin::Pin;

		match &mut *$me {
			$crate::engine::RwFile::Tokio(f, _) => Pin::new(f).$method($($arg),*),
			$crate::engine::RwFile::Sftp(f, _) => Pin::new(f).$method($($arg),*),
			$crate::engine::RwFile::Lua(f) => Pin::new(f).$method($($arg),*),
		}}
	};
}
