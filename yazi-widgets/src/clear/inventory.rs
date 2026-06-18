use ratatui::layout::Rect;

pub struct ClearInventory {
	pub clear: fn(Rect) -> Option<Rect>,
}

inventory::collect!(ClearInventory);
