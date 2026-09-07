#[macro_export]
macro_rules! capability {
	(write) => {
		::yazi_fs::engine::Capabilities::OPEN
	};
	($method:ident) => {
		paste::paste! { ::yazi_fs::engine::Capabilities::[<$method:upper>] }
	};
}
