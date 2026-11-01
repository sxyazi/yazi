var ya-completing = $false
var ya-matcher = $edit:match-prefix~
if (has-key $edit:completion:matcher argument) {
	set ya-matcher = $edit:completion:matcher[argument]
} elif (has-key $edit:completion:matcher '') {
	set ya-matcher = $edit:completion:matcher['']
}

set edit:completion:matcher[argument] = {|seed|
	if $ya-completing {
		set ya-completing = $false
		each {|_| put $true }
	} else {
		$ya-matcher $seed
	}
}

set edit:completion:arg-completer[ya] = {|@words|
	var cands = [(ya cmp elvish -- $@words | from-lines)]
	put $@cands
	set ya-completing = (> (count $cands) 0)
}
