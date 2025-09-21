yazi_macro::mod_flat!(gate metadata read_dir sftp);

pub(super) static CONN: yazi_shared::RoCell<
	parking_lot::Mutex<
		hashbrown::HashMap<
			&'static yazi_vfs::config::ProviderSftp,
			&'static deadpool::managed::Pool<Sftp>,
		>,
	>,
> = yazi_shared::RoCell::new();

pub(super) fn init() { CONN.init(Default::default()); }
