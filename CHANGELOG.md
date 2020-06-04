# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## 0.3.2 (2020/06/04)

- Fix lifetime on Deserializer implementation for Value (thanks @euclio)
- Many documentation fixes (thanks @casey)

## 0.3.1 (2020/05/07)

- Bugfix release allowing generic values to be contained within lists or maps

## 0.3.0 (2020/03/13)

- Added serde support
- Added generic value type that can represent any Bencode value

## 0.2.2 (2020/01/29)

- Make the `no_std` api match the `std` api a little bit more closely.

## 0.2.1 (2019/09/03)

- Add missing [`FromBencode`] implementation for [`BTreeMap`].
- Introduce `std` as default enabled feature.
  - Disabling this feature makes bendy `no_std` compatible.
  - This currently requires that the target provides allocator support and
    also supports `atomic_cas` as bendy contains a default [`ToBencode`]
    implementation for `Arc<T: ToBencode>`.
- Update minimal required rustc version to v1.36 (to use `extern crate alloc`
  inside tests and examples).

## 0.2.0 (2019/02/28)
- Add new `try_into_*` utility methods on [`Object`].
- Introduce ...
  - [`FromBencode`] trait for simpler decoding.
  - a high level encoding [`Error`][`EncodingError`] type.
  - a high level decoding [`Error`][`DecodingError`] type.
  - [`ResultExt`] decoding trait to improve error handling.
- Subscribed into edition 2018 and latest rustfmt version.

**Breaking Changes**

- Remove [`Error`] from the public API.
- Move [`Token`] from [`decoder`] into [`state_tracker`] submodule.
- Rename ...
  - [`encoder`] submodule into [`encoding`].
  - [`decoder`] submodule into [`decoding`].
  - [`Encodable`] trait into [`ToBencode`].
- Changed signatures of all `_or_err` methods on [`Object`] .
- Replaced all occurrences of [`Error`] inside the API with the new high level decoding
  [`Error`][`DecodingError`] and encoding [`Error`][`EncodingError`].

## 0.1.2 (2018/08/14)
- Add [`AsRef<[u8]>`][`AsRef`] and [`From<&[u8]>`][`From`] for [`AsString`] if the content supports them.

## 0.1.1 (2018/08/07)
- Add missing trait derives for the [`AsString`] encoding wrapper.

## 0.1.0 (2018/07/24)
Initial release

<!-- -->

[`AsRef`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
[`AsString`]: https://docs.rs/bendy/latest/bendy/encoding/struct.AsString.html
[`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
[`decoder`]: https://docs.rs/bendy/0.1.2/bendy/decoder/index.html
[`decoding`]: https://docs.rs/bendy/latest/bendy/decoding/index.html
[`DecodingError`]: https://docs.rs/bendy/latest/bendy/decoding/struct.Error.html
[`Encodable`]: https://docs.rs/bendy/0.1.2/bendy/encoder/trait.Encodable.html
[`encoder`]: https://docs.rs/bendy/0.1.2/bendy/encoder/index.html
[`encoding`]: https://docs.rs/bendy/latest/bendy/encoding/index.html
[`EncodingError`]: https://docs.rs/bendy/latest/bendy/encoding/struct.Error.html
[`Error`]: https://docs.rs/bendy/0.1.2/bendy/enum.Error.html
[`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
[`FromBencode`]: https://docs.rs/bendy/latest/bendy/decoding/trait.FromBencode.html
[`Object`]: https://docs.rs/bendy/latest/bendy/decoding/enum.Object.html
[`ResultExt`]: https://docs.rs/bendy/latest/bendy/decoding/trait.ResultExt.html
[`state_tracker`]: https://docs.rs/bendy/latest/bendy/state_tracker/index.html
[`ToBencode`]: https://docs.rs/bendy/latest/bendy/encoding/trait.ToBencode.html
[`Token`]: https://docs.rs/bendy/latest/bendy/state_tracker/enum.Token.html
