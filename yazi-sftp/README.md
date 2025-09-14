# yazi-sftp

A fork of [`russh-sftp`](https://github.com/AspectUnk/russh-sftp) used by Yazi, with some changes:

- Supports paths containing invalid UTF-8
- Supports retrieving file nlink, username, and group
- Uses generic return parameters for a more idiomatic API, e.g.:
  ```rust
  let attrs: responses::Attrs = session.send(requests::Stat::new(path)).await?
  ```
- Reduced dependencies
- Performance optimizations:
  - Copy-on-write for all packets to avoid unnecessary memory allocation
  - Packet lengths are precomputed to avoid secondary allocations
  - Avoids cloning buffers in `AsyncRead` and `AsyncWrite` implementations
