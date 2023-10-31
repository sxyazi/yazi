use std::{future::Future, pin::Pin};

use ratatui::prelude::Rect;

use crate::Position;

pub type InitCompletionType =
	Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = Vec<String>> + Send>> + Send>;
pub type FinishCompletionType = Box<dyn Fn(&str, &str) -> String + Send>;

pub struct InputOpt {
	pub title:             String,
	pub value:             String,
	pub position:          Position,
	pub realtime:          bool,
	pub highlight:         bool,
	pub init_completion:   Option<InitCompletionType>,
	pub finish_completion: Option<FinishCompletionType>,
}

impl InputOpt {
	pub fn top(title: impl AsRef<str>) -> Self {
		Self {
			title:             title.as_ref().to_owned(),
			value:             String::new(),
			position:          Position::Top(
				// TODO: hardcode
				Rect { x: 0, y: 2, width: 50, height: 3 },
			),
			realtime:          false,
			highlight:         false,
			init_completion:   None,
			finish_completion: None,
		}
	}

	pub fn hovered(title: impl AsRef<str>) -> Self {
		Self {
			title:             title.as_ref().to_owned(),
			value:             String::new(),
			position:          Position::Hovered(
				// TODO: hardcode
				Rect { x: 0, y: 1, width: 50, height: 3 },
			),
			realtime:          false,
			highlight:         false,
			init_completion:   None,
			finish_completion: None,
		}
	}

	#[inline]
	pub fn with_value(mut self, value: impl AsRef<str>) -> Self {
		self.value = value.as_ref().to_owned();
		self
	}

	#[inline]
	pub fn with_realtime(mut self) -> Self {
		self.realtime = true;
		self
	}

	#[inline]
	pub fn with_highlight(mut self) -> Self {
		self.highlight = true;
		self
	}

	#[inline]
	pub fn with_completion<
		F1: Fn(String) -> Pin<Box<dyn Future<Output = Vec<String>> + Send>> + Send + 'static,
		F2: Fn(&str, &str) -> String + Send + 'static,
	>(
		mut self,
		init_completion: F1,
		finish_completion: F2,
	) -> Self {
		self.init_completion = Some(Box::new(init_completion));
		self.finish_completion = Some(Box::new(finish_completion));
		self
	}
}
