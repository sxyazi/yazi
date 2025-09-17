yazi_macro::mod_flat!(gate metadata read_dir sftp);

pub(super) static CONN: yazi_shared::RoCell<deadpool::managed::Pool<Sftp>> =
	yazi_shared::RoCell::new();

pub(super) fn init() { CONN.init(deadpool::managed::Pool::builder(Sftp).build().unwrap()); }
