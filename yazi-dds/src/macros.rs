#[macro_export]
macro_rules! body {
	(mgr: $name:ident, $body:expr) => {
		paste::paste! {
			$crate::body::Body::[<Key $name:camel>]($body)
		}
	};
	($layer:ident : $name:ident, $body:expr) => {
		$crate::body::Body::Void(&yazi_parser::VoidOpt)
		// paste::paste! {
		// 	$crate::body::Body::[<Key $layer:camel $name:camel>]($body)
		// }
	};
}
