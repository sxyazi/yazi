use tokio::task;
use yazi_binding::runtime::RuntimeSeed;
use yazi_shared::data::Data;

use crate::{Runner, evaluator::{EvaluateHandle, EvaluateJob}};

impl Runner {
	pub fn evaluate(
		&'static self,
		mut seed: RuntimeSeed,
		bytes: Vec<u8>,
		arg: Data,
	) -> EvaluateHandle {
		let scope = seed.fork();
		let job = EvaluateJob { runner: self, seed, bytes, arg };

		EvaluateHandle::new(scope, task::spawn_blocking(move || job.eval()))
	}
}
