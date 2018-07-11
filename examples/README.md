# Examples of how to use Bendy
This directory contains two examples showcasing how to serialize and deserialize bencoded files
with `bendy`.

* [`decode torrent`](decode_torrent.rs) - deserializes a debian torrent file and prints the deserialized
values except for `pieces`. The `pieces` field contains a hash list which is written to [`pieces.iso`](pieces.iso).

* [`enocde torrent`](encode.torrent.rs) - serializes a torrent file from a struct. The data for the `pieces` field is read
from [`pieces.iso`](pieces.iso).

