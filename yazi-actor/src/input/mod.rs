yazi_macro::mod_flat!(close complete escape recall remember show);

fn remember_history(histories: &mut yazi_core::input::InputHistories, group: &str, value: &str) {
	if histories.remember(group, value) {
		yazi_macro::log_if_err!(yazi_dds::Pubsub::pub_after_history(group, value));
	}
}
