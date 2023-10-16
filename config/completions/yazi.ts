const completion: Fig.Spec = {
  name: "yazi",
  description: "",
  options: [
    {
      name: "--cwd-file",
      description: "Write the cwd on exit to this file",
      isRepeatable: true,
      args: {
        name: "cwd_file",
        isOptional: true,
        template: "filepaths",
      },
    },
    {
      name: "--chooser-file",
      description: "Write the selected files on open emitted by the chooser mode",
      isRepeatable: true,
      args: {
        name: "chooser_file",
        isOptional: true,
        template: "filepaths",
      },
    },
    {
      name: "--clear-cache",
      description: "Clear the cache directory",
    },
    {
      name: ["-h", "--help"],
      description: "Print help",
    },
    {
      name: ["-V", "--version"],
      description: "Print version",
    },
  ],
  args: {
    name: "cwd",
    isOptional: true,
    template: "filepaths",
  },
};

export default completion;
