# Bendy

[![Build Status](https://travis-ci.org/P3KI/bendy.svg?branch=master)](https://travis-ci.org/P3KI/bendy)
[![Current Version](https://meritbadge.herokuapp.com/bendy)](https://crates.io/crates/bendy)
[![License: MIT/Apache-2.0](https://img.shields.io/github/license/P3KI/bendy.svg)](#license)

A Rust library for encoding and decoding bencode with enforced canonicalization rules.
[Bencode](https://en.wikipedia.org/wiki/Bencode) is a simple but very effective encoding
scheme, originating with the BitTorrent peer-to-peer system.

## Known alternatives:
This is not the first library to implement Bencode. In fact there's several implementations
already:

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
data actually is sorted* and complains if it isn't. In the end, once bendy serialized your
data, it's Bencode through and through. So it's perfectly compatible with every other
Bencode library.

Just remember: At this point *only bendy* enforces the correctness of the canonical
format if you read it back in.

### Canonical representation
Bendy ensures that any de-serialize / serialize roundtrip produces the exact *same*
and *correct* binary representation. This is relevant if you're dealing with unordered
sets or map-structured data where theoretically the order is not relevant, but in practice
it is. Especially if you want to ensure that cryptographic signatures related to the data
structure do not get invalidated accidentially.

| Datastructure | Default Impl | Comment                                                                                    |
|---------------|--------------|--------------------------------------------------------------------------------------------|
| Vec           | ✔            | Defines own ordering                                                                       |
| VecDeque      | ✔            | Defines own ordering                                                                       |
| LinkedList    | ✔            | Defines own ordering                                                                       |
| HashMap       | ✔            | Ordering missing but content is ordered by key byte representation.                        |
| BTreeMap      | ✔            | Defines own ordering                                                                       |
| HashSet       | ✘            | (Unordered) Set handling not yet defined                                                   |
| BTreeSet      | ✘            | (Unordered) Set handling not yet defined                                                   |
| BinaryHeap    | ✘            | Ordering missing                                                                           |
| Iterator      | ~            | `emit_unchecked_list()` allows to emit any iterable but user needs to ensure the ordering. |

**Attention:**

- Since most list types already define their inner ordering, datastructures 
  like `Vec`, `VecDeque`, and `LinkedList` will not get sorted during encoding!

- There is no default implementation for handling generic iterators.
  This is by design. `Bendy` cannot tell from an iterator whether the underlying
  structure requires sorting or not and would have to take data as-is.

## Usage

### Optional: Limitiation of recursive parsing

**What?**

The library allows to set an expected recursion depth limit for de- and encoding.
If set, the parser will use this value as an upper limit for the validation of any nested
data structure and abort with an error if an additional level of nesting is detected.

While the encoding limit itself is primarily there to increase the confidence of bendy
users in their own validation code, the decoding limit should be used to avoid
parsing of malformed or malicious external data.

 - The encoding limit can be set through the `MAX_DEPTH`
    field inside any implementation of the `Encodable` trait.
 - The decoding limit can be set through a call of `with_max_depth`
    on the `Decoder` object.
    
**How?**

The nesting level calculation always starts on level zero, is incremented by one when
the parser enters a nested bencode element (i.e. list, dictionary) and decrement as
soon as the related element ends. Therefore any values decoded as bencode strings
or integers do not affect the nesting limit.

### Encoding Bencode
In most cases it should be enough to pass the object to encode into the `emit`
function of the encoder as this will serialize any type implementing the
`Encodable` trait.

Next to `emit` the encoder also provides a list of functions to encode specific
bencode primitives (i.e. `emit_int` and `emit_str`) and nested bencode elements
(i.e. `emit_dict` and `emit_list`). These methods should be used during the
implementation of the `Encodable` trait or if its necessary to output a specific
non default data type.

**Hint:** As its a very common pattern to serialize a `Vec<u8>` as a byte string
Bendy exposes the `AsString` wrapper. This can be used to encapsulate any element
implementing `AsRef<[u8]>` to output itself as a bencode string instead of a list.
For a usage example see the categorie `Encode a byte string`.

#### Encoding an integer

```rust
use bendy::encoder::Encoder;

let mut encoder = Encoder::new();
encoder.emit(1010011010).unwrap();

let output = encoder.get_output().unwrap();
assert_eq!("i1010011010e", std::str::from_utf8(&output).unwrap());
```

#### Encode a byte string

```rust
use bendy::encoder::Encoder;

let mut encoder = Encoder::new();
encoder.emit("foo").unwrap();

let output = encoder.get_output().unwrap();
assert_eq!("3:foo", std::str::from_utf8(&output).unwrap());
```

```rust
use bendy::encoder::{Encoder, AsString};

let byte_vector = vec![0u8, 1, 2];

let mut encoder = Encoder::new();
encoder.emit(AsString(byte_vector)).unwrap();

let output = encoder.get_output().unwrap();
assert_eq!("3:\x00\x01\x02", std::str::from_utf8(&output).unwrap());
```

#### Encode a dictionary

```rust
use bendy::{
    encoder::{Encodable, SingleItemEncoder, Encoder},
    Error as BencodeError,
};

struct Dict{
    bar: String,
}

impl Encodable for Dict{
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"bar", &self.bar)?;
            Ok(())
        })
    }
}

fn main() {
    let dict = Dict { bar: "baz".to_owned() };
    
    let mut encoder = Encoder::new();
    encoder.emit(dict).unwrap();
    
    let output = encoder.get_output().unwrap();
    assert_eq!(
        "d3:bar3:baze",
        std::str::from_utf8(&output).unwrap()
    );
}
```

#### Encode a list

```rust
use bendy::encoder::{Encoder, List};

let list = vec!["foo", "bar", "baz"];

let mut encoder = Encoder::new();
encoder.emit(List(&list)).unwrap();

let output = encoder.get_output().unwrap();
assert_eq!(
    "l3:foo3:bar3:baze",
    std::str::from_utf8(&output).unwrap()
);
```

### Decoding Bencode

#### Decode an integer

```rust
use bendy::decoder::Decoder;

let mut decoder = Decoder::new(b"i1010011010e");
let object = decoder.next_object().unwrap().unwrap();

let number = object.integer_str_or_err(-1).unwrap();
assert_eq!("1010011010", number);
```

#### Decode a byte string

```rust
use bendy::decoder::Decoder;

let mut decoder = Decoder::new(b"11:foo bar baz");
let object = decoder.next_object().unwrap().unwrap();

let bytes =  object.bytes_or_err(-1).unwrap();
assert_eq!("foo bar baz", std::str::from_utf8(&bytes).unwrap());
```

#### Decode a dictionary

```rust
use bendy::decoder::{Decoder, Object};

let mut decoder = Decoder::new(b"d3:foo3:bare");
let object = decoder.next_object().unwrap();

if let Some(Object::Dict(mut dict_decoder)) = object {
    
    if let (b"foo",value) = dict_decoder.next_pair().unwrap().unwrap() {
        let bytes = value.bytes_or_err(-1).unwrap();
        assert_eq!("bar", std::str::from_utf8(&bytes).unwrap());
    }
}
```

#### Decode a list

```rust
use bendy::decoder::{Decoder, Object};

let mut decoder = Decoder::new(b"l3:foo3:bar3:baze");
let object = decoder.next_object().unwrap();
let mut result : Vec<&str> = vec![];

if let Some(Object::List(mut list_decoder)) = object {

    while let Some(list_element) = list_decoder.next_object().unwrap(){
        let bytes =  list_element.bytes_or_err(-1).unwrap();
        result.push(std::str::from_utf8(&bytes).unwrap());
    }
}

assert_eq!(["foo", "bar", "baz"][..], result[..]);
```

## Usage of unsafe code
The parser wouldn't require any unsafe code to work but it still contains a single unsafe call
to `str::from_utf8_unchecked`. This call is used to avoid a duplicated UTF-8 check when the
parser converts the bytes representing an incoming integer into a `&str` after its successful
validation.

*Disclaimer: Further unsafe code may be introduced through the dependency on `failure` and
`failure-derive`.*
