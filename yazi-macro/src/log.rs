#[doc(hidden)]
#[macro_export]
macro_rules! log {
	($level:expr, $($args:tt)+) => {{
		let callsite = $crate::log_callsite!($level);
		if ::yazi_shim::log::private::enabled(callsite) {
			::yazi_shim::log::private::emit(callsite, format_args!($($args)+));
		}
	}};
}

#[macro_export]
macro_rules! log_if_err {
	($expr:expr) => {
		$crate::log_if_err!(stringify!($expr), $expr)
	};
	($label:expr, $expr:expr) => {
		$crate::log_if_err!($expr, "{}", $label)
	};
	($expr:expr, $fmt:expr, $($args:tt)*) => {{
		if let Err(e) = $expr {
			$crate::error!("{} failed: {e}", format_args!($fmt, $($args)*));
		}
	}};
}

#[macro_export]
macro_rules! debug {
	($($args:tt)+) => { $crate::log!(::yazi_shim::log::private::Level::DEBUG, $($args)+) };
}

#[macro_export]
macro_rules! warn {
	($($args:tt)+) => { $crate::log!(::yazi_shim::log::private::Level::WARN, $($args)+) };
}

#[macro_export]
macro_rules! error {
	($($args:tt)+) => { $crate::log!(::yazi_shim::log::private::Level::ERROR, $($args)+) };
}

#[macro_export]
macro_rules! time {
	($expr:expr) => {
		$crate::time!(stringify!($expr), $expr)
	};
	($label:expr, $expr:expr) => {
		$crate::time!($expr, "{}", $label)
	};
	($expr:expr, $fmt:expr, $($args:tt)*) => {{
		if $crate::log_enabled!(::yazi_shim::log::private::Level::DEBUG) {
			let start = std::time::Instant::now();
			let result = $expr;
			$crate::debug!("{} took {:?}", format_args!($fmt, $($args)*), start.elapsed());
			result
		} else {
			$expr
		}
	}};
}

#[doc(hidden)]
#[macro_export]
macro_rules! log_enabled {
	($level:expr) => {
		::yazi_shim::log::private::enabled($crate::log_callsite!($level))
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! log_callsite {
	($level:expr) => {{
		use ::yazi_shim::log::private::{DefaultCallsite, FieldSet, Kind, Metadata, identify_callsite};

		static META: Metadata<'static> = Metadata::new(
			concat!("event ", file!(), ":", line!()),
			module_path!(),
			$level,
			Some(file!()),
			Some(line!()),
			Some(module_path!()),
			FieldSet::new(&["message"], identify_callsite!(&CALLSITE)),
			Kind::EVENT,
		);

		static CALLSITE: DefaultCallsite = DefaultCallsite::new(&META);
		&CALLSITE
	}};
}
