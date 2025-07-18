#[macro_export]
macro_rules! local_or_err {
	($name:literal) => {
		if !$crate::LOCAL.read().contains_key($name) {
			anyhow::bail!("No local event handler found");
		}
	};
}
