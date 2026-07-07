# Proposal: rclone-backed VFS provider (re: #3395)

> **STATUS: LOCAL DRAFT — not posted anywhere.** This is the comment we'd
> put on issue #3395 once we decide to engage upstream. Nothing has been
> pushed or shared.

## Summary

A read-only `rclone://` VFS provider that shells out to the `rclone` CLI,
giving yazi native browsing of any of rclone's 100+ remotes (S3, GCS, Azure,
Backblaze, WebDAV, …) through a single provider — no FUSE/NFS mount. This is
the "integrate rclone" item on the #3395 roadmap, and reuses the VFS
infrastructure from #3821 / the (closed) S3 PR #3843.

A working proof-of-concept is implemented and verified against live GCS and
S3 buckets (directory browsing, metadata, byte-exact streaming reads + seeks).

## URL / scheme shape

```
rclone://<service>//<path>
```

`<service>` is a name defined in `vfs.toml`; it maps to an rclone remote.
One scheme covers every backend — the backend choice lives in rclone's own
config, not in yazi.

```toml
# ~/.config/yazi/vfs.toml
[services.gcs]
type   = "rclone"
remote = "gcs"           # an existing rclone remote (`rclone listremotes`)

[services.s3]
type       = "rclone"
remote     = "s3"
# binary      = "/opt/homebrew/bin/rclone"   # optional
# config_file = "/path/to/rclone.conf"       # optional
# flags       = ["--fast-list"]              # optional extra args
```

Then: `rclone://gcs//my-bucket/some/dir`.

`SchemeKind::Rclone` mirrors `Sftp` — remote, virtual, Unix-path,
byte-encoded filenames — so it threads through the existing scheme/URL enums
with minimal new surface (mostly folded into existing `Sftp` match arms).

## How each operation maps

| Provider op        | rclone                                         |
|--------------------|------------------------------------------------|
| `read_dir`         | `rclone lsjson <remote>:<path>`                |
| `metadata`         | `rclone lsjson --stat <remote>:<path>`         |
| `open`/read        | `rclone cat --offset <pos> <remote>:<path>`    |
| write/create/…     | `Unsupported` (read-only v1)                   |

### The `AsyncSeek`-over-a-CLI problem

`Provider::File` must be `AsyncRead + AsyncSeek + AsyncWrite`. rclone has no
persistent file handle, so the `File` type streams `rclone cat` from the
current offset; a `seek` kills the child process and the next read respawns
`rclone cat --offset <new>`. This makes the file seekable enough for yazi's
previewer and copy paths without any random-access protocol. Verified
byte-exact vs `rclone cat` for both full reads and post-seek tail reads.

## Scope decisions (seeking maintainer input)

1. **Read-only first.** Matches PR #3843's scope. Writes (`rcat`/`copyto`)
   and mutations (`deletefile`, `mkdir`, `moveto`) are a clean follow-up but
   raise consistency/UX questions worth deciding separately.
2. **CLI vs `rclone rcd` daemon.** The PoC spawns a subprocess per op
   (simple, no lifecycle). A long-lived `rclone rcd` + HTTP API would cut
   per-op latency and enable a connection-pool model like the SFTP provider.
   Happy to go either way — which do you prefer?
3. **Metadata.** rclone gives `Size`/`ModTime`/`IsDir` but no
   uid/gid/perm/nlink; the PoC synthesizes `0755`/`0644`. `Capabilities`
   only models `symlink` today, so read-only-ness surfaces as per-op
   `Unsupported`. Want a `writable`/`read_only` capability flag?
4. **Pagination.** `lsjson` returns a whole directory as one JSON array
   (simple, but memory-heavy for very large dirs). Incremental listing would
   need `--files-only`/chunking or the rc daemon. Acceptable for v1?
5. **`search` on remote schemes** is currently disabled (per #3395) — this
   provider doesn't change that.

## Rough edit surface

~40 files, but the vast majority are one-line enum/match additions folded
into existing `Sftp` arms. The real code is the ~6-file provider module in
`yazi-vfs/src/provider/rclone/` plus `config/rclone.rs`. Diffstat of the
PoC: 44 files, ~790 insertions.

## Open question for the maintainer

Before we polish this into a PR: does this scheme shape and the CLI-first
(vs rcd-daemon) approach match what you discussed on Telegram? If the daemon
model is preferred, we'll refactor the transport before submitting.
