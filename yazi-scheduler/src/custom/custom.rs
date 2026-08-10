use std::future;

use super::CustomOut;
use crate::custom::CustomIn;

#[derive(Clone, Copy, Debug, Default)]
pub struct Custom;

impl Custom {
	pub(crate) fn new() -> Self { Self }

	pub(crate) async fn r#do(&self, r#_in: CustomIn) -> Result<(), CustomOut> {
		future::pending().await
	}
}
