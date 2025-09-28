yazi_macro::mod_pub!(sftp);

yazi_macro::mod_flat!(calculator dir_entry gate provider providers read_dir rw_file);

pub(super) fn init() { sftp::init(); }
