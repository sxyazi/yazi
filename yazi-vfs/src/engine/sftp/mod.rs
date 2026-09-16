yazi_macro::mod_flat!(conn demand metadata read_dir sftp);

static CONN: yazi_shim::cell::RoCell<
	parking_lot::Mutex<hashbrown::HashMap<yazi_shared::auth::AuthArc, deadpool::managed::Pool<Conn>>>,
> = yazi_shim::cell::RoCell::new();

pub(super) fn init() {
	CONN.init(Default::default());

	tokio::spawn(async {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(60)).await;

			CONN.lock().retain(|_, pool| {
				pool.retain(|_, metrics| metrics.last_used().as_secs() < 300);

				let status = pool.status();
				status.size > 0 || status.waiting > 0
			});
		}
	});
}
