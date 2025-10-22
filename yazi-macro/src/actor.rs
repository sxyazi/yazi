#[macro_export]
macro_rules! act {
	(@pre $layer:ident : $name:ident, $cx:ident, $opt:ident) => {
		if let Some(hook) = <act!($layer:$name) as yazi_actor::Actor>::hook($cx, &$opt) {
			<act!(core:preflight)>::act($cx, (hook, yazi_dds::spark!($layer:$name, $opt))).map(|spark| spark.try_into().unwrap())
		} else {
			Ok($opt)
		}
	};
	(@impl $layer:ident : $name:ident, $cx:ident, $opt:ident) => {{
		$cx.level += 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.push(concat!(stringify!($layer), ":", stringify!($name)));

		let result = match act!(@pre $layer:$name, $cx, $opt) {
			Ok(opt) => <act!($layer:$name) as yazi_actor::Actor>::act($cx, opt),
			Err(e) => Err(e),
		};

		$cx.level -= 1;
		#[cfg(debug_assertions)]
		$cx.backtrace.pop();

		result
	}};

	($layer:ident : $name:ident, $cx:ident, $cmd:expr) => {
		<act!($layer:$name) as yazi_actor::Actor>::Options::try_from($cmd)
			.map_err(anyhow::Error::from)
			.and_then(|opt| act!(@impl $layer:$name, $cx, opt))
	};
	($layer:ident : $name:ident, $cx:ident) => {
		act!($layer:$name, $cx, <<act!($layer:$name) as yazi_actor::Actor>::Options as Default>::default())
	};
	($layer:ident : $name:ident) => {
		paste::paste! { yazi_actor::$layer::[<$name:camel>] }
	};

	($name:ident, $cx:expr, $cmd:expr) => {
		$cmd.try_into().map_err(anyhow::Error::from).and_then(|opt| $cx.$name(opt))
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
}
