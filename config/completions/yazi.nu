module completions {

  export extern yazi [
    cwd?: string              # Set the current working directory
    --cwd-file: string        # Write the cwd on exit to this file
    --chooser-file: string    # Write the selected files on open emitted by the chooser mode
    --clear-cache             # Clear the cache directory
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

}

export use completions *
