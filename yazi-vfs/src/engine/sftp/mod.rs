yazi_macro::mod_flat!(conn demand metadata read_dir sftp);

static CONN: yazi_shim::cell::RoCell<
	parking_lot::Mutex<
		hashbrown::HashMap<yazi_shared::auth::AuthArc, deadpool::managed::WeakPool<Conn>>,
	>,
> = yazi_shim::cell::RoCell::new();

pub(super) fn init() { CONN.init(Default::default()); }
