# Cross Compile yazi for Different Platforms with Docker

I do not like to have multiple different development dependencies installed on
my computer, so I usually use [Docker](https://www.docker.com) to develop and
build.

Here are some dockerfiles to help in cross-compiling yazi for different
platforms which I use.

- QNAP TS 431 with OS 4.3.6
- macOS

## Build for QNAP TS431

As I own a QNAP TS431, I wanted to run yazi there as well.
Unfortunately it's a quite old system so I doubt many of you will have a use for
this, but maybe it helps someone to adapt it for another, older Linux.

To build simply run

`platform/ts431/build.sh`

When finished, you sholud have a directory `platform/built/ts431` containing
both, `ya` and `yazi`.

## Build for Apple

Here we have some options:

- `-a` `apple`|`intel`
- `-s` *own-sdk-version*
- `-m` *minimal-target-version*

When you have another macOS SDK than version 11.3, you can pass the version
number using `-s VERSIONNUMBER`.
Otherwise 11.3 will be downloaded and used.

To generate an SDK tarball, follow the instructions for
[osxcross](https://github.com/tpoechtrager/osxcross?tab=readme-ov-file#packaging-the-sdk).

Place your `MacOSX${VERSIONNMBER}.sdk.tar.xz` in`platform/mac/`.

Running `platform/mac/build.sh` without any options, will generate executables
for Apple Silicon hardware using SDK 11.3 and targeting at least 11.0.

To compile the same for Intel hardware run `platform/mac/build.sh -a intel`.

The resulting `ya` and `yazi` can be found in `platform/built/mac/apple` for
Apple Silicon and `platform/built/mac/intel` for Intel hardware.
