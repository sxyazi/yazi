use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;


pub struct USERDICT {
	pub exist: bool,
    pub table: HashMap<char, Vec<char>>
}

pub fn read_user_dict(p:&Path) -> USERDICT {
    let mut ret = USERDICT  {
        exist: false,
        table: HashMap::new()
    };

    if let Ok(file) = File::open(p.join("user.dict")) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(entry) = line {
                let parts: Vec<char> = entry.trim().chars().collect();
                if parts.len() == 3 {
                    ret.table.entry(parts[0])
                        .or_insert_with(Vec::new)
                        .push(parts[2]);
                }
            }
        }
        ret.exist = true;
    }
    ret
}
