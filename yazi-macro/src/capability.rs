#[macro_export]
macro_rules! capability {
	(write) => {
		|c: ::yazi_fs::engine::Capabilities| c.open
	};
	(reroute) => {
		|c: ::yazi_fs::engine::Capabilities| c.reroute != 0
	};
	($method:ident) => {
		|c: ::yazi_fs::engine::Capabilities| c.$method
	};
}
