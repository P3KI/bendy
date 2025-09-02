# pretty-bencode

[![Current Version](https://meritbadge.herokuapp.com/bendy)](https://crates.io/crates/pretty-bencode)
[![License: BSD-3-Clause](https://img.shields.io/github/license/P3KI/bendy.svg)](https://github.com/P3KI/bendy/blob/master/LICENSE-BSD3)

A small CLI program to pretty print bencode. Just adds indentation and makes ascii-only byte strings print in a readable form. Built using the [bendy](https://github.com/P3KI/bendy) crate.

Will only ingest valid bencode.

# Installation

`cargo install pretty-bencode`

# Usage

Can take a single bencode unit (list, dict, integer or byte string) from stdin.

```
echo "li1ei2ee" | pretty-bencode
l
    i1e
    i2e
e
```

Can take any number of bencode units, each from a different file.

```bash
pretty-bencode file1 file2 file3
```

Call with the `--string-literal` (`-s`) flag to print it formatted as a
rust string literal.

```
echo "li1ei2ee" | pretty-bencode -s
let pretty_bencode = b"\
l\
        i1e\
        i2e\
e\
"
```

# Byte String Representation

If bytestrings contain only bytes that are printable ASCII (bytes in range `40 <= b < 127`) they will be printed as ascii characters. Otherwise they will be printed in hexadecimal. E.g. a byte string containing only one byte with the value zero will be printed as `1:\x00`. The `1:` is of course the standard bencode byte string length prefix.
