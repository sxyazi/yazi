
use builtin;
use str;

set edit:completion:arg-completer[yazi] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'yazi'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'yazi'= {
            cand --cwd-file 'Write the cwd on exit to this file'
            cand --chooser-file 'Write the selected files on open emitted by the chooser mode'
            cand --clear-cache 'Clear the cache directory'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
