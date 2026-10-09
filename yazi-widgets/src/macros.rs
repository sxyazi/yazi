macro_rules! act {
	($name:ident, $cx:expr, $action:expr) => {
		yazi_shared::event::FromAction::from_action($action, &*$cx).and_then(|opt| $cx.$name(opt))
	};
	($name:ident, $cx:expr) => {
		$cx.$name(Default::default())
	};
}
