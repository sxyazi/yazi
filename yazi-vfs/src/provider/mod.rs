yazi_macro::mod_pub!(rclone sftp);

yazi_macro::mod_flat!(calculator copier dir_entry gate lua provider providers read_dir rw_file);

pub(super) fn init() { sftp::init(); }
