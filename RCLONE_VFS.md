# rclone VFS provider for yazi

Browse **any of rclone's 100+ remotes** — S3, GCS, Azure, Backblaze, SFTP,
WebDAV, Google Drive, … — natively inside [yazi](https://github.com/sxyazi/yazi),
with no FUSE/NFS mount. It rides on yazi's existing VFS layer (the same
infrastructure behind the built-in SFTP provider) and shells out to the `rclone`
binary you already have.

Addresses the "Integrate rclone" item on #3395. Read-only for now.

---

## Usage

Point a VFS service at an existing rclone remote in `~/.config/yazi/vfs.toml`:

```toml
[services.gcs]
type   = "rclone"
remote = "gcs"          # any remote from `rclone listremotes`

[services.s3]
type       = "rclone"
remote     = "s3"
# binary      = "/opt/homebrew/bin/rclone"   # optional: path to rclone
# config_file = "/path/to/rclone.conf"       # optional: non-default config
# flags       = ["--fast-list"]              # optional: extra rclone args
```

Then browse with a `rclone://<service>//<path>` URL:

```
rclone://gcs//my-bucket/some/dir
rclone://s3//my-bucket
```

`rclone://gcs//` (empty path) lists the remote's buckets. Everything else is
normal yazi navigation, preview, and copy-out.

## What works

- **Directory listing** — `rclone lsjson`
- **Metadata** — `rclone lsjson --stat` (size + mtime; POSIX mode is synthesized
  since object stores have no permissions)
- **Reading / preview / copy-out** — a seekable file handle that streams
  `rclone cat --offset`
- **Object stores and filesystem backends** — `remote:/path` keeps its leading
  slash, so `local`, `sftp`, etc. work alongside S3/GCS/…

## What doesn't (yet)

- **Writes** — every mutating op returns `Unsupported`. Read-only by design for a
  first cut; uploads (`rclone rcat`/`copyto`) and mutations are a clean follow-up.
- **Remote search** — still disabled globally for remote schemes (per #3395).

## How it works

yazi's provider dispatch is a set of static enums, so a new `rclone://` scheme
threads a `SchemeKind::Rclone` variant through the scheme/URL type system —
mostly folded into the existing `Sftp` arms, since rclone remotes share the same
traits (remote, virtual, Unix-style paths).

The interesting part is the file handle. rclone has no persistent open-file
concept, so `Provider::File` (which must be `AsyncRead + AsyncSeek`) is
implemented by streaming `rclone cat` from the current position; a **seek kills
the child process and the next read respawns `rclone cat --offset <new>`**. This
is enough for yazi's previewer and for its parallel chunked copier (which opens
several handles and seeks each to its 8 MiB slice).

## Testing

Two layers, no credentials required to run CI:

- **`tests/rclone_local.rs` — hermetic.** Points an rclone `local` remote at a
  generated temp directory, so `lsjson`/`cat` run against the filesystem. Needs
  only the `rclone` binary; **skips itself if rclone isn't installed** (so CI
  without rclone stays green). Covers listing, read, seek, missing-object, and a
  >8 MiB parallel multi-chunk copy with position-varying bytes so any offset slip
  is caught.
- **`tests/rclone_read.rs` — live, `#[ignore]`d.** Runs against a real remote you
  point it at via env vars (`YAZI_RCLONE_TEST_SMALL` / `_BIG` / `_DIR`) — nothing
  is hard-coded.

Verified end-to-end against live **GCS and S3** buckets: byte-exact reads,
mid-file seeks, and both simple and progressive (parallel multi-chunk) copy-out.

## Notes / open questions for upstream

- **CLI vs daemon.** This spawns an `rclone` process per operation — simple, no
  lifecycle to manage. A long-lived `rclone rcd` (remote-control daemon) would
  cut per-op latency and enable connection pooling like the SFTP provider does.
  Happy to go either way.
- **Read-only scope.** Matches the earlier S3 PR (#3843). Writes can follow once
  the direction is settled.
