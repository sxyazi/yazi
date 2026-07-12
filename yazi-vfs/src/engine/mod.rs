yazi_macro::mod_pub!(lua sftp);

yazi_macro::mod_flat!(absolute calculator copier dir_entry demand engine engines read_dir rw_file);

pub(super) fn init() { sftp::init(); }
