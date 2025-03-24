use super::*;

const DIGITS_VALUES: [&str; 100] = [
	"000", "001", "002", "003", "004", "005", "006", "007", "008", "009", "010", "011", "012", "013",
	"014", "015", "016", "017", "018", "019", "020", "021", "022", "023", "024", "025", "026", "027",
	"028", "029", "030", "031", "032", "033", "034", "035", "036", "037", "038", "039", "040", "041",
	"042", "043", "044", "045", "046", "047", "048", "049", "050", "051", "052", "053", "054", "055",
	"056", "057", "058", "059", "060", "061", "062", "063", "064", "065", "066", "067", "068", "069",
	"070", "071", "072", "073", "074", "075", "076", "077", "078", "079", "080", "081", "082", "083",
	"084", "085", "086", "087", "088", "089", "090", "091", "092", "093", "094", "095", "096", "097",
	"098", "099",
];

#[test]
fn test_digits_advance_100_iterations() {
	let mut buf = String::new();
	let counter = Digits;
	for (idx, &expected_value) in DIGITS_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32, 3, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in DIGITS_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32));
	}
}

const UPPERCASE_ANSI_VALUES: [&str; 100] = [
	"A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
	"T", "U", "V", "W", "X", "Y", "Z", "AA", "AB", "AC", "AD", "AE", "AF", "AG", "AH", "AI", "AJ",
	"AK", "AL", "AM", "AN", "AO", "AP", "AQ", "AR", "AS", "AT", "AU", "AV", "AW", "AX", "AY", "AZ",
	"BA", "BB", "BC", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BK", "BL", "BM", "BN", "BO", "BP",
	"BQ", "BR", "BS", "BT", "BU", "BV", "BW", "BX", "BY", "BZ", "CA", "CB", "CC", "CD", "CE", "CF",
	"CG", "CH", "CI", "CJ", "CK", "CL", "CM", "CN", "CO", "CP", "CQ", "CR", "CS", "CT", "CU", "CV",
];

const LOWERCASE_ANSI_VALUES: [&str; 100] = [
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
	"t", "u", "v", "w", "x", "y", "z", "aa", "ab", "ac", "ad", "ae", "af", "ag", "ah", "ai", "aj",
	"ak", "al", "am", "an", "ao", "ap", "aq", "ar", "as", "at", "au", "av", "aw", "ax", "ay", "az",
	"ba", "bb", "bc", "bd", "be", "bf", "bg", "bh", "bi", "bj", "bk", "bl", "bm", "bn", "bo", "bp",
	"bq", "br", "bs", "bt", "bu", "bv", "bw", "bx", "by", "bz", "ca", "cb", "cc", "cd", "ce", "cf",
	"cg", "ch", "ci", "cj", "ck", "cl", "cm", "cn", "co", "cp", "cq", "cr", "cs", "ct", "cu", "cv",
];

#[test]
fn test_ansi_upper_advance_100_iterations() {
	let mut buf = String::new();
	let counter = AnsiUpper;
	for (idx, &expected_value) in UPPERCASE_ANSI_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in UPPERCASE_ANSI_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}

#[test]
fn test_ansi_lower_advance_100_iterations() {
	let mut buf = String::new();
	let counter = AnsiLower;
	for (idx, &expected_value) in LOWERCASE_ANSI_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in LOWERCASE_ANSI_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}

const UPPERCASE_ROMAN_VALUES: [&str; 100] = [
	"I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV", "XV",
	"XVI", "XVII", "XVIII", "XIX", "XX", "XXI", "XXII", "XXIII", "XXIV", "XXV", "XXVI", "XXVII",
	"XXVIII", "XXIX", "XXX", "XXXI", "XXXII", "XXXIII", "XXXIV", "XXXV", "XXXVI", "XXXVII",
	"XXXVIII", "XXXIX", "XL", "XLI", "XLII", "XLIII", "XLIV", "XLV", "XLVI", "XLVII", "XLVIII",
	"XLIX", "L", "LI", "LII", "LIII", "LIV", "LV", "LVI", "LVII", "LVIII", "LIX", "LX", "LXI",
	"LXII", "LXIII", "LXIV", "LXV", "LXVI", "LXVII", "LXVIII", "LXIX", "LXX", "LXXI", "LXXII",
	"LXXIII", "LXXIV", "LXXV", "LXXVI", "LXXVII", "LXXVIII", "LXXIX", "LXXX", "LXXXI", "LXXXII",
	"LXXXIII", "LXXXIV", "LXXXV", "LXXXVI", "LXXXVII", "LXXXVIII", "LXXXIX", "XC", "XCI", "XCII",
	"XCIII", "XCIV", "XCV", "XCVI", "XCVII", "XCVIII", "XCIX", "C",
];

const LOWERCASE_ROMAN_VALUES: [&str; 100] = [
	"i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii", "xiv", "xv",
	"xvi", "xvii", "xviii", "xix", "xx", "xxi", "xxii", "xxiii", "xxiv", "xxv", "xxvi", "xxvii",
	"xxviii", "xxix", "xxx", "xxxi", "xxxii", "xxxiii", "xxxiv", "xxxv", "xxxvi", "xxxvii",
	"xxxviii", "xxxix", "xl", "xli", "xlii", "xliii", "xliv", "xlv", "xlvi", "xlvii", "xlviii",
	"xlix", "l", "li", "lii", "liii", "liv", "lv", "lvi", "lvii", "lviii", "lix", "lx", "lxi",
	"lxii", "lxiii", "lxiv", "lxv", "lxvi", "lxvii", "lxviii", "lxix", "lxx", "lxxi", "lxxii",
	"lxxiii", "lxxiv", "lxxv", "lxxvi", "lxxvii", "lxxviii", "lxxix", "lxxx", "lxxxi", "lxxxii",
	"lxxxiii", "lxxxiv", "lxxxv", "lxxxvi", "lxxxvii", "lxxxviii", "lxxxix", "xc", "xci", "xcii",
	"xciii", "xciv", "xcv", "xcvi", "xcvii", "xcviii", "xcix", "c",
];

#[test]
fn test_roman_upper_advance_100_iterations() {
	let mut buf = String::new();
	let counter = RomanUpper;
	for (idx, &expected_value) in UPPERCASE_ROMAN_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in UPPERCASE_ROMAN_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}

#[test]
fn test_roman_lower_advance_100_iterations() {
	let mut buf = String::new();
	let counter = RomanLower;
	for (idx, &expected_value) in LOWERCASE_ROMAN_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in LOWERCASE_ROMAN_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}

const UPPERCASE_CYRILLIC_VALUES: [&str; 100] = [
	"А", "Б", "В", "Г", "Д", "Е", "Ж", "З", "И", "К", "Л", "М", "Н", "О", "П", "Р", "С", "Т", "У",
	"Ф", "Х", "Ц", "Ч", "Ш", "Щ", "Э", "Ю", "Я", "АА", "АБ", "АВ", "АГ", "АД", "АЕ", "АЖ", "АЗ",
	"АИ", "АК", "АЛ", "АМ", "АН", "АО", "АП", "АР", "АС", "АТ", "АУ", "АФ", "АХ", "АЦ", "АЧ", "АШ",
	"АЩ", "АЭ", "АЮ", "АЯ", "БА", "ББ", "БВ", "БГ", "БД", "БЕ", "БЖ", "БЗ", "БИ", "БК", "БЛ", "БМ",
	"БН", "БО", "БП", "БР", "БС", "БТ", "БУ", "БФ", "БХ", "БЦ", "БЧ", "БШ", "БЩ", "БЭ", "БЮ", "БЯ",
	"ВА", "ВБ", "ВВ", "ВГ", "ВД", "ВЕ", "ВЖ", "ВЗ", "ВИ", "ВК", "ВЛ", "ВМ", "ВН", "ВО", "ВП", "ВР",
];

const LOWERCASE_CYRILLIC_VALUES: [&str; 100] = [
	"а", "б", "в", "г", "д", "е", "ж", "з", "и", "к", "л", "м", "н", "о", "п", "р", "с", "т", "у",
	"ф", "х", "ц", "ч", "ш", "щ", "э", "ю", "я", "аа", "аб", "ав", "аг", "ад", "ае", "аж", "аз",
	"аи", "ак", "ал", "ам", "ан", "ао", "ап", "ар", "ас", "ат", "ау", "аф", "ах", "ац", "ач", "аш",
	"ащ", "аэ", "аю", "ая", "ба", "бб", "бв", "бг", "бд", "бе", "бж", "бз", "би", "бк", "бл", "бм",
	"бн", "бо", "бп", "бр", "бс", "бт", "бу", "бф", "бх", "бц", "бч", "бш", "бщ", "бэ", "бю", "бя",
	"ва", "вб", "вв", "вг", "вд", "ве", "вж", "вз", "ви", "вк", "вл", "вм", "вн", "во", "вп", "вр",
];

#[test]
fn test_cyrillic_upper_advance_100_iterations() {
	let mut buf = String::new();
	let counter = CyrillicUpper;
	for (idx, &expected_value) in UPPERCASE_CYRILLIC_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in UPPERCASE_CYRILLIC_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}

#[test]
fn test_cyrillic_lower_advance_100_iterations() {
	let mut buf = String::new();
	let counter = CyrillicLower;
	for (idx, &expected_value) in LOWERCASE_CYRILLIC_VALUES.iter().enumerate() {
		let _ = counter.value_to_buffer(idx as u32 + 1, 1, &mut buf);
		assert_eq!(expected_value, &buf);
		buf.clear();
	}

	for (idx, &expected_value) in LOWERCASE_CYRILLIC_VALUES.iter().enumerate() {
		assert_eq!(counter.string_to_value(expected_value), Some(idx as u32 + 1));
	}
}
