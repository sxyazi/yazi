#[macro_export]
macro_rules! act {
	(@pre $layer:ident : $name:ident, $cx:ident, $opt:ident) => {
		<act!($layer:$name) as yazi_actor::Actor>::hook($cx, &$opt).map(|hook| {
			<act!(core:preflight) as yazi_actor::Actor>::act($cx, (hook, yazi_dds::body!($layer:$name, &$opt)))
		})
	};
	(@impl $layer:ident : $name:ident, $cx:ident, $opt:ident) => {{
		$cx.level += 1;
		let result = match act!(@pre $layer:$name, $cx, $opt) {
			Some(Ok(yazi_shared::event::Data::Boolean(true))) => Err(anyhow::anyhow!("canceled on preflight")),
			None | Some(Ok(_)) => <act!($layer:$name) as yazi_actor::Actor>::act($cx, $opt),
			Some(e @ Err(_)) => e,
		};
		$cx.level -= 1;
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
