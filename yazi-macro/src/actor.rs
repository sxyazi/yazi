#[macro_export]
macro_rules! act {
	($layer:ident : $name:ident, $cx:expr, $cmd:expr) => {{
		#[allow(unused_imports)]
		use ::yazi_actor::Actor;
		<act!($layer:$name)>::act($cx, $cmd.try_into()?)
	}};
	($layer:ident : $name:ident, $cx:expr) => {{
		#[allow(unused_imports)]
		use yazi_actor::Actor;
		<act!($layer:$name)>::act($cx, Default::default())
	}};
	($name:ident, $cx:expr, $cmd:expr) => {
		$cx.$name($cmd.try_into()?)
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
	($layer:ident : $name:ident) => {
		paste::paste! { yazi_actor::$layer::[<$name:camel>] }
	};
}
