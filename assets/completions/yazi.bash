_yazi_bash_words() {
	local LC_ALL=C
	local line=${COMP_LINE:0:COMP_POINT} word= quote= char
	local started=0 i start=$((COMP_POINT - ${#1}))

	words=()
	prefix= quoting=
	for ((i = 0; i < ${#line}; i++)); do
		if ((i == start)); then prefix=$word; quoting=$quote; fi
		char=${line:i:1}
		if [[ $quote == "'" ]]; then
			if [[ $char == "'" ]]; then quote=; continue; fi
		elif [[ $char == \\ ]]; then
			((i++))
			char=${line:i:1}
			[[ $char == $'\n' ]] && continue
			if [[ $quote == '"' ]]; then
				case $char in
					'$'|'`'|'"'|\\|'') ;;
					*) word+='\' ;;
				esac
			fi
		elif [[ $quote == '"' ]]; then
			case $char in
				'"') quote=; continue ;;
				'$'|'`'|'!') return 1 ;;
			esac
		else
			case $char in
				' '|$'\t')
					((started)) && words+=("$word")
					word= started=0
					continue
					;;
				"'"|'"') quote=$char; started=1; continue ;;
				'$'|'`'|';'|'|'|'&'|'<'|'>'|'('|')'|$'\n'|'*'|'?'|'['|'{'|'!') return 1 ;;
				'~')
					[[ -n $word && $word != *= && $word != *: ]] || return 1
					;;
				'#')
					((started)) || return 1
					;;
			esac
		fi

		word+=$char
		started=1
	done
	if ((start == ${#line})); then prefix=$word; quoting=$quote; fi
	words+=("$word")
}

_yazi() {
	local -a words
	local prefix quoting cand char

	COMPREPLY=()
	_yazi_bash_words "$2" || return 0
	compopt +o nospace 2>/dev/null || :

	while IFS= read -r -d '' cand; do
		[[ -n $cand && $cand == "$prefix"* ]] || continue
		if [[ ${#COMPREPLY[@]} -eq 0 && $cand == *[/=:] ]]; then
			compopt -o nospace 2>/dev/null || :
		fi

		cand=${cand#"$prefix"}
		case $quoting in
			"'") cand=${cand//\'/\'\\\'\'} ;;
			'"')
				for char in \\ '"' '$' '`'; do
					cand=${cand//"$char"/\\$char}
				done
				;;
			*) printf -v cand %q "$cand" ;;
		esac
		COMPREPLY+=("$cand")
	done < <(
		yazi --completion=bash -- "${words[@]}"
	)
}

complete -o nospace -o bashdefault -o default -o nosort -F _yazi yazi 2>/dev/null ||
	complete -o nospace -o bashdefault -o default -F _yazi yazi
