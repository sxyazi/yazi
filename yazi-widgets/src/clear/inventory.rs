use ratatui_core::layout::Rect;

pub struct ClearInventory {
	pub clear: fn(Rect) -> Option<Rect>,
}

inventory::collect!(ClearInventory);
