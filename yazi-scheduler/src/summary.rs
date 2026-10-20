use ordered_float::OrderedFloat;
use serde::Serialize;
use yazi_macro::impl_data_any;

use crate::{Ongoing, Progress};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TaskSummary {
	pub total_tasks:            u32,
	pub(crate) succeeded_tasks: u32,
	pub(crate) failed_tasks:    u32,
	pub(crate) total_items:     u64,
	pub(crate) succeeded_items: u64,
	pub(crate) failed_items:    u64,
	pub(crate) percent:         Option<OrderedFloat<f32>>,
}

impl_data_any!(TaskSummary);

impl From<&Ongoing> for TaskSummary {
	fn from(value: &Ongoing) -> Self {
		let mut summary = Self::default();
		let mut percent_sum = 0.0f64;
		let mut percent_count = 0;

		for task in value.values() {
			let s: Self = task.prog.into();
			if s.total_tasks == 0 {
				continue;
			}

			summary.total_tasks += 1;
			summary.total_items = summary.total_items.saturating_add(s.total_items);
			summary.succeeded_items = summary.succeeded_items.saturating_add(s.succeeded_items);
			summary.failed_items = summary.failed_items.saturating_add(s.failed_items);

			if let Some(p) = s.percent {
				percent_sum += p.0 as f64;
				percent_count += 1;
			}

			if task.prog.running() {
				continue;
			} else if task.prog.success() {
				summary.succeeded_tasks += 1;
			} else {
				summary.failed_tasks += 1;
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
