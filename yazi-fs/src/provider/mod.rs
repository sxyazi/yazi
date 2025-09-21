yazi_macro::mod_pub!(local sftp);

yazi_macro::mod_flat!(calculator dir_entry gate provider providers read_dir rw_file traits);

pub fn init() { sftp::init(); }
