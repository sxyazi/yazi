use super::*;

// Helper to quickly compare Ok(Vec<Tuple>)
fn assert_ok_paths(result: Result<Vec<Tuple>, NameGenerationErrors>, expected: &[&str]) {
	match result {
		Ok(paths) => {
			let actual: Vec<_> = paths.iter().map(|p| p.to_string_lossy()).collect();
			let expected: Vec<_> = expected.iter().copied().map(String::from).collect();
			assert_eq!(actual, expected, "Expected {:?}, got {:?}", expected, actual);
		}
		Err(errs) => panic!("Expected Ok(...), got Err({:?})", errs),
	}
}

#[test]
fn test_generate_names_no_counters() {
	// All lines are just plain filenames (no counters)
	let mut input = "file1.txt\nfile2.txt\nanother_file\n".lines();
	let result = generate_names(&mut input);
	// Should succeed, returning the same lines as PathBuf
	assert_ok_paths(result, &["file1.txt", "file2.txt", "another_file"]);
}

#[test]
fn test_generate_names() {
	let input = [
		// Start = 1, Step = 1, Width = 1
		"file_%{D}_%{d}_%{N}_%{n}_%{A}_%{a}_%{R}_%{r}_%{C}_%{c}.txt", // print 1
		"file_%{D}_%{d}_%{N}_%{n}_%{A}_%{a}_%{R}_%{r}_%{C}_%{c}.txt", // print 1
		// Start = 5, Step = 1, Width = 1
		"file_%{D,5}_%{d,5}_%{N,5}_%{n,5}_%{A,5}_%{a,5}_%{R,5}_%{r,5}_%{C,5}_%{c,5}.txt", // print 5
		"file_%{D  }_%{d  }_%{N  }_%{n  }_%{A  }_%{a  }_%{R  }_%{r  }_%{C  }_%{c  }.txt", // print 6
		// Start = 5 (two times), Step = 1, Width = 1
		"file_%{D,5}_%{d,5}_%{N,5}_%{n,5}_%{A,5}_%{a,5}_%{R,5}_%{r,5}_%{C,5}_%{c,5}.txt", // print 5
		"file_%{D,5}_%{d,5}_%{N,5}_%{n,5}_%{A,5}_%{a,5}_%{R,5}_%{r,5}_%{C,5}_%{c,5}.txt", // print 5 (again)
		// Start = 5, Step = 3, Width = 1
		"file_%{D,_,3}_%{d,_,3}_%{N,_,3}_%{n,_,3}_%{A,_,3}_%{a,_,3}_%{R,_,3}_%{r,_,3}_%{C,_,3}_%{c,_,3}.txt", // print 6
		"file_%{D}_%{d}_%{N}_%{n}_%{A}_%{a}_%{R}_%{r}_%{C}_%{c}.txt", // print 9
		// Start = 5, Step = 3, Width = 3
		"file_%{D,_,_,3}_%{d,_,_,3}_%{N,_,_,3}_%{n,_,_,3}_%{A,_,_,3}_%{a,_,_,3}_%{R,_,_,3}_%{r,_,_,3}_%{C,_,_,3}_%{c,_,_,3}.txt", // print 012
		"file_%{D}_%{d}_%{N}_%{n}_%{A}_%{a}_%{R}_%{r}_%{C}_%{c}.txt", // print 015
		// Change counter formats, Start = 5, Step = 3, Width = 3
		"file_%{A}_%{R}_%{C}_%{N}_%{a}_%{r}_%{c}_%{n}_%{D}_%{d}.txt", // print 018
	]
	.join("\n");

	let result = generate_names(&mut input.lines());
	assert_ok_paths(result, &[
		"file_1_1_1_1_A_a_I_i_А_а.txt",
		"file_2_2_2_2_B_b_II_ii_Б_б.txt",
		"file_5_5_5_5_E_e_V_v_Д_д.txt",
		"file_6_6_6_6_F_f_VI_vi_Е_е.txt",
		"file_5_5_5_5_E_e_V_v_Д_д.txt",
		"file_5_5_5_5_E_e_V_v_Д_д.txt",
		"file_6_6_6_6_F_f_VI_vi_Е_е.txt",
		"file_9_9_9_9_I_i_IX_ix_И_и.txt",
		"file_012_012_012_012_00L_00l_XII_xii_00М_00м.txt",
		"file_015_015_015_015_00O_00o_0XV_0xv_00П_00п.txt",
		"file_00R_XVIII_00Т_018_00r_xviii_00т_018_018_018.txt",
	]);
}

#[test]
fn test_generate_names_mismatch_counters() {
	// First line has 2 counters, second line has 1
	let input = "\
        file_%{n}_%{a}.txt\n\
        file_%{n}.txt\
    ";
	let result = generate_names(&mut input.lines()).unwrap_err();
	// Should produce PathGenError::MismatchCounters
	assert_eq!(result.errors, &[NameGenError::MismatchCounters {
		expected:    2,
		got:         1,
		line_number: 2,
		content:     "file_%{n}.txt",
	}]);
}

#[test]
fn test_generate_names_parse_errors() {
	let input = "\
        Ü-Wagen examplé_слово_%{???}.txt\n\
        Ü-Wagen examplé_слово_%{n,???}.txt\n\
        Ü-Wagen examplé_слово_%{n,1,???}.txt\n\
        Ü-Wagen examplé_слово_%{n,1,1,???}.txt\n\
        Ü-Wagen examplé_слово_%{n,1,1,1,???}.txt\n\
        Ü-Wagen examplé_слово_%{n,1,1,1,}.txt\n\
        Ü-Wagen examplé_слово_%{n}.txt\n\
        Ü-Wagen examplé_слово_%{n}_%{n}.txt
    ";
	let output = generate_names(&mut input.lines()).unwrap_err().to_string();

	let expected = "\
Error: Unexpected counter kind
 |
1| Ü-Wagen examplé_слово_%{???}.txt
 |                         ^^^ Expected: 'one of D, d, N, n, A, a, R, r, C, c', found: '???'

Error: Invalid digit found in string
 |
2| Ü-Wagen examplé_слово_%{n,???}.txt
 |                           ^^^ Expected: 'digit', found: '???'

Error: Invalid digit found in string
 |
3| Ü-Wagen examplé_слово_%{n,1,???}.txt
 |                             ^^^ Expected: 'digit', found: '???'

Error: Invalid digit found in string
 |
4| Ü-Wagen examplé_слово_%{n,1,1,???}.txt
 |                               ^^^ Expected: 'digit', found: '???'

Error: Extra arguments
 |
5| Ü-Wagen examplé_слово_%{n,1,1,1,???}.txt
 |                                ^^^^ Expected: 'no additional arguments'

Error: Extra arguments
 |
6| Ü-Wagen examplé_слово_%{n,1,1,1,}.txt
 |                                ^ Expected: 'no additional arguments'

Error: Mismatch counter numbers
 |
8| Ü-Wagen examplé_слово_%{n}_%{n}.txt
 | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected 1 counters, but got 2

";

	assert_eq!(output, expected);
}
