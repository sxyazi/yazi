#[macro_export]
macro_rules! act {
	(@pre $layer:ident : $name:ident, $cx:ident, $opt:ident) => {
		if let Some(hook) = <$crate::act!($layer:$name) as yazi_actor::Actor>::hook($cx, &$opt) {
			<$crate::act!(core:preflight)>::act($cx, (hook, yazi_parser::spark!($layer:$name, $opt))).map(|spark| spark.try_into().unwrap())
		} else {
			Ok($opt)
		}
	};
	(@impl $layer:ident : $name:ident, $cx:ident, $opt:ident) => {{
		$cx.level += 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.push(concat!(stringify!($layer), ":", stringify!($name)));

		let result = match $crate::act!(@pre $layer:$name, $cx, $opt) {
			Ok(opt) => <$crate::act!($layer:$name) as yazi_actor::Actor>::act($cx, opt),
			Err(e) => Err(e),
		};

		$cx.level -= 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.pop();

		result
	}};

	($layer:ident : $name:ident, $cx:ident, $action:expr) => {
		<$crate::act!($layer:$name) as yazi_actor::Actor>::Form::try_from($action)
			.map_err(anyhow::Error::from)
			.and_then(|opt| $crate::act!(@impl $layer:$name, $cx, opt))
	};
	($layer:ident : $name:ident, $cx:ident) => {
		$crate::act!($layer:$name, $cx, <<$crate::act!($layer:$name) as yazi_actor::Actor>::Form as Default>::default())
	};
	($layer:ident : $name:ident) => {
		paste::paste! { yazi_actor::$layer::[<$name:camel>] }
	};

	($name:ident, $cx:expr, $action:expr) => {
		$action.try_into().map_err(anyhow::Error::from).and_then(|opt| $cx.$name(opt))
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
}
