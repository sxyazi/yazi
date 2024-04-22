use std::{ffi::OsStr, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::event::Cmd;
use yazi_config::USER_DIC;

pub struct Filter {
	raw:   String,
	regex: Regex,
    chars: Vec<char>,
}

struct MatchResult {
    ismatch: bool,
    range: Option<Range<usize>>
}


fn table_match(needle: &Vec<char>, haystack: Vec<char>, flag:bool) -> MatchResult {
    let needle_len = needle.len();
    let haystack_len = haystack.len();
    if needle_len > haystack_len { return MatchResult{ismatch:false, range:None} }

    for i in 0..=(haystack_len - needle_len) {
        let mut found = true;
        for j in 0..needle_len {
            if haystack[i + j] == needle[j] { continue; }
            if let Some(value) = USER_DIC.table.get(&haystack[i+j]) {
                if !value.contains(&needle[j]) {
                    found = false;
                    break;
                }
            } else {
                found = false;
                break;
            }
        }
        if found {
            if flag {
                let start:usize = haystack[0..i].into_iter().collect::<String>().len();
                let end:usize = haystack[0..(i + needle.len())].into_iter().collect::<String>().len();
                return MatchResult{ismatch:true, range:Some(start..end)}
            } else {
                return MatchResult{ismatch:true, range:None}
            }
        }
    }
    MatchResult{ismatch:false, range:None}
}


impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}


impl Filter {
	pub fn new(s: &str, case: FilterCase) -> Result<Self> {
		let regex = match case {
			FilterCase::Smart => {
				let uppercase = s.chars().any(|c| c.is_uppercase());
				RegexBuilder::new(s).case_insensitive(!uppercase).build()?
			}
			FilterCase::Sensitive => Regex::new(s)?,
			FilterCase::Insensitive => RegexBuilder::new(s).case_insensitive(true).build()?,
		};
        let chars: Vec<char> = if USER_DIC.exist { s.chars().collect()} else { Vec::new() };

		Ok(Self { raw: s.to_owned(), regex, chars})
	}

	#[inline]
	pub fn matches(&self, name: &OsStr) -> bool {
        if self.regex.is_match(name.as_encoded_bytes()) {
            return true
        } else if !USER_DIC.exist {
            return false
        } else {
            let name_bytes = name.as_encoded_bytes();
            if let Ok(s) = std::str::from_utf8(name_bytes) {
                return table_match(&self.chars, s.chars().collect(), false).ismatch
            }
            false
        }
    }

	#[inline]
	pub fn highlighted(&self, name: &OsStr) -> Option<Vec<Range<usize>>> {
		let m = self.regex.find(name.as_encoded_bytes());
        return match m {
            Some(r) => Some(vec![r.range()]),
            None =>  {
                let name_bytes = name.as_encoded_bytes();
                return match std::str::from_utf8(name_bytes) {
                    Ok(s) => return match table_match(&self.chars, s.chars().collect(), true).range {
                        Some(r) => Some(vec![r]),
                        None => None
                    },
                    Err(_) => None
                }
            },
        }
	}
}

#[derive(Default, PartialEq, Eq)]
pub enum FilterCase {
	Smart,
	#[default]
	Sensitive,
	Insensitive,
}

impl From<&Cmd> for FilterCase {
	fn from(c: &Cmd) -> Self {
		match (c.bool("smart"), c.bool("insensitive")) {
			(true, _) => Self::Smart,
			(_, false) => Self::Sensitive,
			(_, true) => Self::Insensitive,
		}
	}
}
