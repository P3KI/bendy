# Bendy

[![Build Status](https://travis-ci.org/P3KI/bendy.svg?branch=master)](https://travis-ci.org/P3KI/bendy)
[![Current Version](https://meritbadge.herokuapp.com/bendy)](https://crates.io/crates/bendy)
[![License: BSD-3-Clause](https://img.shields.io/github/license/P3KI/bendy.svg)](https://github.com/P3KI/bendy/blob/master/LICENSE-BSD3)

A Rust library for encoding and decoding bencode with enforced canonicalization rules.
[Bencode](https://en.wikipedia.org/wiki/Bencode) is a simple but very effective encoding
scheme, originating with the BitTorrent peer-to-peer system.

---

You may be looking for:

- [Known Alternatives](#known-alternatives)
- [Why should I use it](#why-should-i-use-it)
- [Usage](#usage)
 - [Encoding](#encoding-with-tobencode)
 - [Decoding](#decoding-with-frombencode)
- [Unsafe Code](#usage-of-unsafe-code)
- [Contributing](#contributing)

---

## Known alternatives:
This is not the first library to implement Bencode. In fact there's several
implementations already:

- Toby Padilla [serde-bencode](https://github.com/toby/serde-bencode)
- Arjan Topolovec's [rust-bencode](https://github.com/arjantop/rust-bencode),
- Murarth's [bencode](https://github.com/murarth/bencode),
- and Jonas Hermsmeier's [rust-bencode](https://github.com/jhermsmeier/rust-bencode)

## Why should I use it?
So why the extra work adding yet-another-version of a thing that already exists, you
might ask?

### Enforced correctness
Implementing a canonical encoding form is straight forward. It comes down to defining
*a proper way of handling unordered data*. The next step is that bendy's sorting data
before encoding it using the regular Bencode rules. If your data is already sorted bendy
will of course skip the extra sorting step to gain efficiency.
But bendy goes a step further to *ensure correctness*: If you hand the library data that
you say is already sorted, bendy still does an in-place verification to *ensure that your
data actually is sorted* and complains if it isn't. In the end, once bendy serialized
your data, it's Bencode through and through. So it's perfectly compatible with every
other Bencode library.

Just remember: At this point *only bendy* enforces the correctness of the canonical
format if you read it back in.

### Canonical representation
Bendy ensures that any de-serialize / serialize round trip produces the exact *same*
and *correct* binary representation. This is relevant if you're dealing with unordered
sets or map-structured data where theoretically the order is not relevant, but in
practice it is, especially if you want to ensure that cryptographic signatures related
to the data structure do not get invalidated accidentally.

| Data Structure | Default Impl | Comment                                                                                    |
|----------------|--------------|--------------------------------------------------------------------------------------------|
| Vec            | ✔            | Defines own ordering                                                                       |
| VecDeque       | ✔            | Defines own ordering                                                                       |
| LinkedList     | ✔            | Defines own ordering                                                                       |
| HashMap        | ✔            | Ordering missing but content is ordered by key byte representation.                        |
| BTreeMap       | ✔            | Defines own ordering                                                                       |
| HashSet        | ✘            | (Unordered) Set handling not yet defined                                                   |
| BTreeSet       | ✘            | (Unordered) Set handling not yet defined                                                   |
| BinaryHeap     | ✘            | Ordering missing                                                                           |
| Iterator       | ~            | `emit_unchecked_list()` allows to emit any iterable but user needs to ensure the ordering. |

**Attention:**

- Since most list types already define their inner ordering, data structures
  like `Vec`, `VecDeque`, and `LinkedList` will not get sorted during encoding!

- There is no default implementation for handling generic iterators.
  This is by design. `Bendy` cannot tell from an iterator whether the underlying
  structure requires sorting or not and would have to take data as-is.

## Usage

First you need to add bendy as a project dependency:

```toml
[dependencies]
bendy = "^0.3"
```

### Encoding with `ToBencode`

To encode an object of a type which already implements the `ToBencode` trait
it is enough to import the trait and call the `to_bencode()` function on the object.

```rust
use bendy::encoding::{ToBencode, Error};

fn main() {}

#[test]
fn encode_vector() -> Result<(), Error> {
    let my_data = vec!["hello", "world"];
    let encoded = my_data.to_bencode()?;

    assert_eq!(b"l5:hello5:worlde", encoded.as_slice());
    Ok(())
}
```

### Implementing `ToBencode`

In most cases it should be enough to overwrite the associated `encode` function
and keep the default implementation of `to_bencode`.

The function will provide you with a `SingleItemEncoder` which must be used to
emit any relevant components of the current object. As long as these implement
`ToBencode` themselves it is enough to pass them into the `emit` function of
the encoder as this will serialize any type implementing the trait.

Next to `emit` the encoder also provides a list of functions to encode specific
bencode primitives (i.e. `emit_int` and `emit_str`) and nested bencode elements
(i.e. `emit_dict` and `emit_list`). These methods should be used if its necessary
to output a specific non default data type.

**Implementing Integer Encoding**

As bencode has native integer support bendy provides default implementations for
all of rusts native integer types. This allows to call `to_bencode` on any integer
object and to pass these objects into the encoder's `emit_int` function.

```rust
use bendy::encoding::{ToBencode, SingleItemEncoder, Error};

struct IntegerWrapper(i64);

impl ToBencode for IntegerWrapper {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_int(self.0)
    }
}

fn main() {}

#[test]
fn encode_integer() -> Result<(), Error> {
    let example = IntegerWrapper(21);

    let encoded = example.to_bencode()?;
    assert_eq!(b"i21e", encoded.as_slice());

    let encoded = 21.to_bencode()?;
    assert_eq!(b"i21e", encoded.as_slice());

    Ok(())
}
```

**Encode a byte string**

Another data type bencode natively supports are byte strings. Therefore bendy
provides default implementations for `String` and `&str`.

```rust
use bendy::encoding::{ToBencode, SingleItemEncoder, Error};

struct StringWrapper(String);

impl ToBencode for StringWrapper {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_str(&self.0)
    }
}

fn main() {}

#[test]
fn encode_string() -> Result<(), Error> {
    let example = StringWrapper("content".to_string());

    let encoded = example.to_bencode()?;
    assert_eq!(b"7:content", encoded.as_slice());

    let encoded = "content".to_bencode()?;
    assert_eq!(b"7:content", encoded.as_slice());

    Ok(())
}
```

As its a very common pattern to represent a byte string as `Vec<u8>` bendy
exposes the `AsString` wrapper. This can be used to encapsulate any element
implementing `AsRef<[u8]>` to output itself as a bencode string instead of a
list.

```rust
use bendy::encoding::{ToBencode, SingleItemEncoder, Error, AsString};

struct ByteStringWrapper(Vec<u8>);

impl ToBencode for ByteStringWrapper {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        let content = AsString(&self.0);
        encoder.emit(&content)
    }
}

fn main() {}

#[test]
fn encode_byte_string() -> Result<(), Error> {
    let example = ByteStringWrapper(b"content".to_vec());

    let encoded = example.to_bencode()?;
    assert_eq!(b"7:content", encoded.as_slice());

    let encoded = AsString(b"content").to_bencode()?;
    assert_eq!(b"7:content", encoded.as_slice());

    Ok(())
}
```

**Encode a dictionary**

If a data structure contains key-value pairs its most likely a good idea to
encode it as a bencode dictionary. This is also true for most structs with
more then one member as it might be helpful to represent their names to ensure
the existence of specific (optional) member.

__Attention:__ To ensure a canonical representation bendy requires that the keys
of a dictionary emitted via `emit_dict` are sorted in ascending order or the
encoding will fail with an error of kind `UnsortedKeys`. In case of an unsorted
dictionary it might be useful to use `emit_and_sort_dict` instead.

```rust
use bendy::encoding::{ToBencode, SingleItemEncoder, Error};

struct Example {
    label: String,
    counter: u64,
}

impl ToBencode for Example {
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"counter", &self.counter)?;
            e.emit_pair(b"label", &self.label)?;

            Ok(())
        })
    }
}

fn main() {}

#[test]
fn encode_dictionary() -> Result<(), Error> {
    let example = Example { label: "Example".to_string(), counter: 0 };

    let encoded = example.to_bencode()?;
    assert_eq!(b"d7:counteri0e5:label7:Examplee", encoded.as_slice());

    Ok(())
}
```

**Encode a list**

While encoding a list bendy assumes the elements inside this list are
inherently sorted through their position inside the list. The implementation
is therefore free to choose its own sorting.

```rust
use bendy::encoding::{ToBencode, SingleItemEncoder, Error};

struct Location(i64, i64);

impl ToBencode for Location {
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), Error> {
        encoder.emit_list(|e| {
            e.emit_int(self.0)?;
            e.emit_int(self.1)
        })
    }
}

fn main() {}

#[test]
fn encode_list() -> Result<(), Error> {
    let example = Location(2, 3);

    let encoded = example.to_bencode()?;
    assert_eq!(b"li2ei3ee", encoded.as_slice());

    Ok(())
}
```

### Decoding with `FromBencode`

To decode an object of a type which already implements the `FromBencode` trait
it is enough to import the trait and call the `from_bencode()` function on the object.

```rust
use bendy::decoding::{FromBencode, Error};

fn main() {}

#[test]
fn decode_vector() -> Result<(), Error> {
    let encoded = b"l5:hello5:worlde".to_vec();
    let decoded = Vec::<String>::from_bencode(&encoded)?;

    assert_eq!(vec!["hello", "world"], decoded);
    Ok(())
}

```

### Implementing `FromBencode`

In most cases it should be enough to overwrite the associated
`decode_bencode_object` function and keep the default implementation of
`from_bencode`.

The function will provide you with an representation of a bencode `Object`
which must be processed to receive any relevant components of the expected data
type. As long as these implement `FromBencode` themselves it is enough to call
`decode_bencode_object` on the expected data type of the element as this will
deserialize any type implementing the trait.

Next to `from_bencode` the bencode `Object` representation also provides a list
of helper functions to itself into specific bencode primitives and container
(i.e. `bytes_or`, `integer_or_else` or `try_into_list`). Which than can be used
to restore the actual element.

**Decode an integer**

As bencode has native integer support bendy provides default implementations
for all of rusts native integer types. This allows to call `from_bencode` on
any type of integer.

*Attention:* If it's necessary to handle a big integer which has no
representation through one of the default data types it's always possible to
access the string version of the number during decoding.

```rust
use bendy::decoding::{FromBencode, Object, Error};

#[derive(Debug, Eq, PartialEq)]
struct IntegerWrapper(i64);

impl FromBencode for IntegerWrapper {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, Error> {
        // This is an example for content handling. It would also be possible
        // to call  `i64::decode_bencode_object(object)` directly.
        let content = object.try_into_integer()?;
        let number = content.parse::<i64>()?;

        Ok(IntegerWrapper(number))
    }
}

fn main() {}

#[test]
fn decode_integer() -> Result<(), Error> {
    let encoded = b"i21e".to_vec();

    let example = IntegerWrapper::from_bencode(&encoded)?;
    assert_eq!(IntegerWrapper(21), example);

    let example = i64::from_bencode(&encoded)?;
    assert_eq!(21, example);

    Ok(())
}
```

**Decode a byte string**

In most cases it is possible to restore a string from its bencode
representation as a byte sequence via the `String::from_utf8` and
`str::from_utf8`.

```rust
use bendy::decoding::{FromBencode, Object, Error};

#[derive(Debug, Eq, PartialEq)]
struct StringWrapper(String);

impl FromBencode for StringWrapper {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, Error> {
        // This is an example for content handling. It would also be possible
        // to call  `String::decode_bencode_object(object)` directly.
        let content = object.try_into_bytes()?;
        let content = String::from_utf8(content.to_vec())?;

        Ok(StringWrapper(content))
    }
}

fn main() {}

#[test]
fn decode_string() -> Result<(), Error> {
    let encoded = b"7:content".to_vec();

    let example = StringWrapper::from_bencode(&encoded)?;
    assert_eq!(StringWrapper("content".to_string()), example);

    let example = String::from_bencode(&encoded)?;
    assert_eq!("content".to_string(), example);

    Ok(())
}
```

If the content is a non utf8 encoded string or an actual byte sequence the
`AsString` wrapper might be useful to restore the bencode string object as
a sequence of bytes through an object of type `Vec<u8>`.

```rust
use bendy::{
    decoding::{FromBencode, Object, Error},
    encoding::AsString,
};

#[derive(Debug, Eq, PartialEq)]
struct ByteStringWrapper(Vec<u8>);

impl FromBencode for ByteStringWrapper {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, Error> {
        let content = AsString::decode_bencode_object(object)?;
        Ok(ByteStringWrapper(content.0))
    }
}

fn main() {}

#[test]
fn decode_byte_string() -> Result<(), Error> {
    let encoded = b"7:content".to_vec();

    let example = ByteStringWrapper::from_bencode(&encoded)?;
    assert_eq!(ByteStringWrapper(b"content".to_vec()), example);

    let example = AsString::from_bencode(&encoded)?;
    assert_eq!(b"content".to_vec(), example.0);

    Ok(())
}
```

**Decode a dictionary**

Unwrapping the bencode object into a dictionary will provide a dictionary
decoder which can be used to access the included key-value pairs.

To improve the error handling in case of huge or multiple nested dictionaries
the decoding module provides a `ResultExt` trait which allows to add a context
description in case of an error. If multiple context calls are nested they will
concatenated in a dot notation like style.

```rust
use bendy::decoding::{FromBencode, Object, Error, ResultExt};

#[derive(Debug, Eq, PartialEq)]
struct Example {
    label: String,
    counter: u64,
}

impl FromBencode for Example {
    const EXPECTED_RECURSION_DEPTH: usize = 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error> {
        let mut counter = None;
        let mut label = None;

        let mut dict = object.try_into_dictionary()?;
        while let Some(pair) = dict.next_pair()? {
            match pair {
                (b"counter", value) => {
                    counter = u64::decode_bencode_object(value)
                        .context("counter")
                        .map(Some)?;
                },
                (b"label", value) => {
                    label = String::decode_bencode_object(value)
                        .context("label")
                        .map(Some)?;
                },
                (unknown_field, _) => {
                    return Err(Error::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                },
            }
        }

        let counter = counter.ok_or_else(|| Error::missing_field("counter"))?;
        let label= label.ok_or_else(|| Error::missing_field("label"))?;

        Ok(Example { counter, label })
    }
}

fn main() {}

#[test]
fn decode_dictionary() -> Result<(), Error> {
    let encoded = b"d7:counteri0e5:label7:Examplee".to_vec();
    let expected = Example { label: "Example".to_string(), counter: 0 };

    let example = Example::from_bencode(&encoded)?;
    assert_eq!(expected, example);

    Ok(())
}
```

**Decode a list**

Unwrapping the bencode object into a list will provide a list decoder which can
be used to access the contained elements.

```rust
use bendy::decoding::{FromBencode, Object, Error};

#[derive(Debug, PartialEq, Eq)]
struct Location(i64, i64);

impl FromBencode for Location {
    const EXPECTED_RECURSION_DEPTH: usize = 1;

    fn decode_bencode_object(object: Object) -> Result<Self, Error> {
        let mut list = object.try_into_list()?;

        let x = list.next_object()?.ok_or(Error::missing_field("x"))?;
        let x = i64::decode_bencode_object(x)?;

        let y = list.next_object()?.ok_or(Error::missing_field("y"))?;
        let y = i64::decode_bencode_object(y)?;

        Ok(Location(x, y))
    }
}

fn main() {}

#[test]
fn decode_list() -> Result<(), Error> {
    let encoded = b"li2ei3ee".to_vec();
    let expected = Location(2, 3);

    let example = Location::from_bencode(&encoded)?;
    assert_eq!(expected, example);

    Ok(())
}
```

### Optional: Limitation of recursive parsing

**What?**

The library allows to set an expected recursion depth limit for de- and encoding.
If set, the parser will use this value as an upper limit for the validation of any nested
data structure and abort with an error if an additional level of nesting is detected.

While the encoding limit itself is primarily there to increase the confidence of bendy
users in their own validation code, the decoding limit should be used to avoid
parsing of malformed or malicious external data.

 - The encoding limit can be set through the `MAX_DEPTH` constant in any implementation
   of the `ToBencode` trait.
 - The decoding limit can be set through the `EXPECTED_RECURSION_DEPTH` constant in any
   implementation of the `FromBencode` trait.

**How?**

The nesting level calculation always starts on level zero, is incremented by one when
the parser enters a nested bencode element (i.e. list, dictionary) and decrement as
soon as the related element ends. Therefore any values decoded as bencode strings
or integers do not affect the nesting limit.

### Serde Support

Bendy supports serde when the `serde` feature is enabled:

```toml
[dependencies]
bendy = { version = "^0.3", features = ["std", "serde"] }
serde = { version = "1.0", features = ["derive"] }
```

With the feature enabled, values can be serialized to and deserialized from
bencode with `bendy::serde::from_bytes` and `bendy::serde::to_bytes`
respectively:


```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Foo {
    bar: String,
}

fn main() {
    let value = Foo {
        bar: "hello".into(),
    };
    let bencode = bendy::serde::to_bytes(&value).unwrap();
    assert_eq!(bencode, b"d3:bar5:helloe");
    let deserialized = bendy::serde::from_bytes::<Foo>(&bencode).unwrap();
    assert_eq!(deserialized, value);
}
```

Information on how Rust types are represented in bencode is available in the
[serde module documentation](https://docs.rs/bendy/*/bendy/serde/index.html).

## Usage of unsafe code
The parser would not require any unsafe code to work but it still contains a single unsafe call
to `str::from_utf8_unchecked`. This call is used to avoid a duplicated UTF-8 check when the
parser converts the bytes representing an incoming integer into a `&str` after its successful
validation.

*Disclaimer: Further unsafe code may be introduced through the dependency on the `failure` crate.*

## Contributing

We welcome everyone to ask questions, open issues or provide merge requests.
Each merge request will be reviewed and either landed in the main tree or given
feedback for changes that would be required.

All code in this repository is under the [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause)
license.
