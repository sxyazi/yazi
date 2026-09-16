macro_rules! act {
	($name:ident, $cx:expr, $action:expr) => {
		$action.try_into().map_err(anyhow::Error::from).and_then(|opt| $cx.$name(opt))
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
}
