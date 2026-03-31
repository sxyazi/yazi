use yazi_shared::strand::StrandBuf;

#[derive(Debug, Clone)]
pub struct CmpItem {
	pub name:   StrandBuf,
	pub is_dir: bool,
}
