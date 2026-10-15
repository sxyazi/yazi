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
	($me:expr, $method:ident $args:tt) => {
		poll_rw!($me, $method$args; Local, Sftp, Lua)
	};
	($me:expr, $method:ident $args:tt; $($variant:ident),+ $(,)?) => {
		match &mut *$me {
			$(Self::$variant(f) => std::pin::Pin::new(f).$method$args,)+
		}
	};
}
