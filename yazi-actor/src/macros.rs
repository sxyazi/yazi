#[macro_export]
macro_rules! act {
	(@pre $layer:ident : $name:ident, $cx:ident, $opt:ident) => {
		if let Some(hook) = <$crate::act!($layer:$name) as $crate::Actor>::hook($cx, &$opt) {
			<$crate::act!(core:preflight)>::act($cx, (hook, $crate::yazi_parser::spark!($layer:$name, $opt))).map(|spark| spark.try_into().unwrap())
		} else {
			Ok($opt)
		}
	};
	(@impl $layer:ident : $name:ident, $cx:ident, $opt:ident) => {{
		$cx.level += 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.push(concat!(stringify!($layer), ":", stringify!($name)));

		let result = match $crate::act!(@pre $layer:$name, $cx, $opt) {
			Ok(opt) => <$crate::act!($layer:$name) as $crate::Actor>::act($cx, opt),
			Err(e) => Err(e),
		};

		$cx.level -= 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.pop();

		result
	}};

	($layer:ident : $name:ident, $cx:ident, $action:expr) => {
		<$crate::act!($layer:$name) as $crate::Actor>::Form::try_from($action)
			.map_err($crate::anyhow::Error::from)
			.and_then(|opt| $crate::act!(@impl $layer:$name, $cx, opt))
	};
	($layer:ident : $name:ident, $cx:ident) => {
		$crate::act!($layer:$name, $cx, <<$crate::act!($layer:$name) as $crate::Actor>::Form as Default>::default())
	};
	($layer:ident : $name:ident) => {
		$crate::paste::paste! { $crate::$layer::[<$name:camel>] }
	};

	($name:ident, $cx:expr, $action:expr) => {
		$action.try_into().map_err($crate::anyhow::Error::from).and_then(|opt| $cx.$name(opt))
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
}

#[macro_export]
macro_rules! confirm {
	($cx:ident, $cfg:expr) => {{
		let token = yazi_shared::CompletionToken::default();
		match $crate::act!(confirm:show, $cx, $crate::yazi_parser::confirm::ShowForm { cfg: $cfg, token: token.clone() }) {
			Ok(_) => Ok(token),
			Err(e) => Err(e)
		}
	}};
}

#[macro_export]
macro_rules! input {
	($cx:ident, $opt:expr) => {{
		use yazi_dds::Pubsub;
		use yazi_shim::strum::IntoStr;

		let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
		let opt = $opt.with_cb(move |event| {
			yazi_macro::log_if_err!(Pubsub::pub_after_input((&event).into_str(), event.value()));
			tx.send(event).ok();
		});

		match $crate::act!(input:show, $cx, opt) {
			Ok(_) => Ok(rx),
			Err(e) => Err(e)
		}
	}};
}
