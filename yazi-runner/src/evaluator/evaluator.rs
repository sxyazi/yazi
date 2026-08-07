use compact_str::CompactString;
use tokio::task;
use yazi_binding::Scope;
use yazi_shared::data::Data;

use crate::{Runner, evaluator::{EvaluateHandle, EvaluateJob}};

impl Runner {
	pub fn evaluate(
		&'static self,
		name: CompactString,
		scope: Scope,
		bytes: Vec<u8>,
		arg: Data,
	) -> EvaluateHandle {
		let scope = scope.child();
		let job = EvaluateJob { runner: self, scope: scope.clone(), name, bytes, arg };

		EvaluateHandle::new(scope, task::spawn_blocking(move || job.eval()))
	}
}
