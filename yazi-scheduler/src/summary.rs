use ordered_float::OrderedFloat;
use serde::Serialize;

use crate::{Ongoing, Progress};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TaskSummary {
	pub total:   u32,
	pub success: u32,
	pub failed:  u32,
	pub percent: Option<OrderedFloat<f32>>,
}

impl From<&Ongoing> for TaskSummary {
	fn from(value: &Ongoing) -> Self {
		let mut summary = Self::default();
		let mut percent_sum = 0.0f64;
		let mut percent_count = 0;

		for task in value.values() {
			let s: Self = task.prog.into();
			if s.total == 0 && !task.prog.failed() {
				continue;
			}

			summary.total += 1;
			if let Some(p) = s.percent {
				percent_sum += p.0 as f64;
				percent_count += 1;
			}

			if task.prog.running() {
				continue;
			} else if task.prog.success() {
				summary.success += 1;
			} else {
				summary.failed += 1;
			}
		}

		summary.percent = if percent_count == 0 {
			None
		} else {
			Some(OrderedFloat((percent_sum / percent_count as f64) as f32))
		};
		summary
	}
}
