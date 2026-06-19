use ratatui_widgets::block::Padding;

pub trait Padable {
	fn padding(self, pad: impl Into<Padding>) -> Self;
}

impl Padable for ratatui_core::layout::Rect {
	fn padding(self, padding: impl Into<Padding>) -> Self {
		let p = padding.into();
		let horizontal = p.left.saturating_add(p.right);
		let vertical = p.top.saturating_add(p.bottom);

		Self {
			x:      self.x.saturating_add(p.left),
			y:      self.y.saturating_add(p.top),
			width:  self.width.saturating_sub(horizontal),
			height: self.height.saturating_sub(vertical),
		}
	}
}
