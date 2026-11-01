var yazi-completing = $false
var yazi-matcher = $edit:match-prefix~
if (has-key $edit:completion:matcher argument) {
	set yazi-matcher = $edit:completion:matcher[argument]
} elif (has-key $edit:completion:matcher '') {
	set yazi-matcher = $edit:completion:matcher['']
}

set edit:completion:matcher[argument] = {|seed|
	if $yazi-completing {
		set yazi-completing = $false
		each {|_| put $true }
	} else {
		$yazi-matcher $seed
	}
}

set edit:completion:arg-completer[yazi] = {|@words|
	var cands = [(yazi --completion=elvish -- $@words | from-lines)]
	put $@cands
	set yazi-completing = (> (count $cands) 0)
}
