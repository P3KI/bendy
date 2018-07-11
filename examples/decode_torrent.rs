//! A decoder for torrent files
//!
//! This decoder reads a torrent file and deserialize the bencode formatted information
//! into an object.
//!
//! You can run this example with:
//!
//!     cargo run --example decode_torrent

extern crate bendy;
extern crate failure;

#[macro_use]
extern crate failure_derive;

use std::str::from_utf8;

use bendy::{
    decoder::{Decoder, DictDecoder, Object},
    Error as BencodeError,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Fail)]
pub enum Error {
    /// Read Item was not conforming to torrent file standard
    #[fail(display = "Read Item is unknown: {}", _0)]
    UnknownField(String),
    /// Necessary Fields are missing
    #[fail(display = "A necessary field is missing: {}", _0)]
    MissingField(String),
    /// BencodeError Wrapper
    #[fail(display = "Decoding failed.")]
    DecodeError(#[cause] BencodeError),
    /// Torrent file is empty, incomplete or flawed
    #[fail(display = "Torrent file is empty, incomplete or flawed.")]
    TorrentStructureError,
    /// Data field not parsable or wrong datatype.
    ///
    /// Expects two Strings:
    ///  1).The expected datatype and
    ///  2) the torrent field for which the data was parsed to give
    ///     the user more data for debugging.
    #[fail(
        display = "Malformed field. Expected {} in field: {}.",
        _0,
        _1
    )]
    MalFormedField(String, String),
}

impl Error {
    fn unknown_field(field_name: &str) -> Error {
        Error::UnknownField(field_name.into())
    }

    fn missing_field(field_name: &str) -> Error {
        Error::MissingField(field_name.into())
    }

    fn mal_formed_field(ex_type: &str, field: &str) -> Error {
        Error::MalFormedField(ex_type.into(), field.into())
    }
}

impl From<BencodeError> for Error {
    fn from(err: BencodeError) -> Self {
        Error::DecodeError(err)
    }
}

/// Main struct containing all torrent information. All fields containing an option are
/// not in the torrent file standard but are not yet approved extensions to it.
/// The used extensions are only included to be able to decode the whole Debian torrent file.
///
/// To keep the example simple we won't parse the bencode integers into actual number types as
/// the bencode integer definition is actually a bigint and the content may nit fit in a specific
/// one.
struct Metainfo {
    pub announce: String,
    pub info: Info,
    pub comment: Option<String>,         // not official element
    pub creation_date: Option<String>,   // not official element
    pub http_seeds: Option<Vec<String>>, // not official element
}

struct Info {
    pub piece_length: String,
    pub pieces: Vec<u8>,
    pub name: String,
    pub file_length: String,
}

/// Treats object as bencode integer. On success the field is stored String.
fn decode_integer_as_string(field_name: &str, data: Object) -> Result<String, Error> {
    let error = || Error::mal_formed_field("Integer", field_name);

    let number_string = data.integer_str_or_else_err(error)?;
    Ok(number_string.to_owned())
}

/// Treats object as byte string. On success the byte string is converted to String.
fn decode_bytes_as_string(field_name: &str, data: Object) -> Result<String, Error> {
    let error = || Error::mal_formed_field("String", field_name);

    let bytes = data.bytes_or_else_err(error)?;
    let text = from_utf8(bytes).map_err(|_| error())?;

    Ok(text.to_owned())
}

/// Treats object as byte string. On success the byte string is converted to Vector.
fn decode_bytes_as_vec(field_name: &str, data: Object) -> Result<Vec<u8>, Error> {
    let result = data.bytes_or_err(Error::mal_formed_field("String", field_name))?;

    Ok(result.to_vec())
}

/// Treats object as List of Strings.
///
/// On success the List is parsed for Strings. Any String found is written to a Vector.
/// A non empty List is then return as `Ok(Some(list))`. An empty List is allowed and
/// returned as `Ok(None)`.
fn decode_list_of_strings(field_name: &str, data: Object) -> Result<Option<Vec<String>>, Error> {
    let mut list = Vec::new();
    let mut list_element;
    let mut list_dec = data.list_or_err(Error::mal_formed_field("List", field_name))?;

    while let Some(object) = list_dec.next_object()? {
        list_element = decode_bytes_as_string(field_name, object)?;
        list.push(list_element);
    }

    if list.is_empty() {
        Ok(None)
    } else {
        Ok(Some(list))
    }
}

/// Treats object as dictionary containing all fields fo the info struct.
/// On success the dictionary is parsed for the fields of info which are
/// necessary for torrent. Any missing field will result in a missing field
/// error which will stop the decoding.
fn decode_info(field_name: &str, data: Object) -> Result<Option<Info>, Error> {
    let mut builder_file_length: Option<String> = None;
    let mut builder_name: Option<String> = None;
    let mut builder_piece_length: Option<String> = None;
    let mut builder_pieces: Option<Vec<u8>> = None;

    let mut dict_dec =
        data.dictionary_or_err(Error::mal_formed_field("Info Dictionary", field_name))?;

    while let Some(pair) = dict_dec.next_pair()? {
        match pair {
            (b"length", value) => {
                let file_length = decode_integer_as_string("torrent.info.file_length", value)?;
                builder_file_length = Some(file_length);
            }
            (b"name", value) => {
                let name = decode_bytes_as_string("torrent.info.name", value)?;
                builder_name = Some(name);
            }
            (b"piece length", value) => {
                let piece_length = decode_integer_as_string("torrent.info.length", value)?;
                builder_piece_length = Some(piece_length);
            }
            (b"pieces", value) => {
                let pieces = decode_bytes_as_vec("torrent.info.pieces", value)?;
                builder_pieces = Some(pieces);
            }
            (unknown_field, _) => {
                return match from_utf8(unknown_field) {
                    Ok(field) => Err(Error::unknown_field(field)),
                    Err(_) => Err(Error::TorrentStructureError),
                }
            }
        }
    }

    // Check if all necessary fields are there.
    let info = Info {
        file_length: builder_file_length.ok_or_else(|| Error::missing_field("file_length"))?,
        name: builder_name.ok_or_else(|| Error::missing_field("name"))?,
        piece_length: builder_piece_length.ok_or_else(|| Error::missing_field("piece_length"))?,
        pieces: builder_pieces.ok_or_else(|| Error::missing_field("pieces"))?,
    };

    Ok(Some(info))
}

/// Entry point for decoding a torrent. The dictionary is parsed for all non-optional and optional fields.
/// Missing optional fields are ignored but any other missing fields result in stopping the decoding and in spawning an `MissingField` error.
fn decode_torrent(mut dict_dec: DictDecoder) -> Result<Metainfo, Error> {
    let mut builder_announce: Option<String> = None;
    let mut builder_comment: Option<String> = None;
    let mut builder_creation_date: Option<String> = None;
    let mut builder_https_seed: Option<Vec<String>> = None;
    let mut builder_info: Option<Info> = None;

    while let Some(pair) = dict_dec.next_pair()? {
        match pair {
            (b"announce", value) => {
                let announce = decode_bytes_as_string("torrent.announce", value)?;
                builder_announce = Some(announce);
            }
            (b"comment", value) => {
                let comment = decode_bytes_as_string("torrent.comment", value)?;
                builder_comment = Some(comment);
            }
            (b"creation date", value) => {
                let creation_date = decode_integer_as_string("torrent.creation_date", value)?;
                builder_creation_date = Some(creation_date);
            }
            (b"httpseeds", value) => {
                builder_https_seed = decode_list_of_strings("torrent.http_seeds", value)?
            }
            (b"info", value) => builder_info = decode_info("torrent.info", value)?,
            (unknown_field, _) => {
                return match from_utf8(unknown_field) {
                    Ok(field) => Err(Error::unknown_field(field)),
                    Err(_) => Err(Error::TorrentStructureError),
                };
            }
        }
    }

    let meta_info = Metainfo {
        announce: builder_announce.ok_or_else(|| Error::missing_field("announce"))?,
        info: builder_info.ok_or_else(|| Error::missing_field("info"))?,
        comment: builder_comment,
        creation_date: builder_creation_date,
        http_seeds: builder_https_seed,
    };

    Ok(meta_info)
}

static EXAMPLE_TORRENT: &[u8] =
    include_bytes!("torrent_files/debian-9.4.0-amd64-netinst.iso.torrent");

fn main() -> Result<(), failure::Error> {
    // max_depth is three because the deepest structure is dictionary of dictionary of list
    let mut decoder = Decoder::new(EXAMPLE_TORRENT).with_max_depth(2);

    // check if EXAMPLE_TORRENT contains a dictionary, the entry structure of a bencoded torrent
    let torrent = match decoder.next_object()? {
        Some(Object::Dict(dict)) => decode_torrent(dict)?,
        None | _ => {
            eprint!("Non-parsable file: Expected bencode dictionary.");
            return Err(Error::TorrentStructureError)?;
        }
    };

    println!("announce: {}", torrent.announce);
    println!("name: {}", torrent.info.name);
    println!("length: {}", torrent.info.piece_length);

    let comment = torrent
        .comment
        .unwrap_or_else(|| "Optional field \"comment\" is empty.".to_owned());
    println!("comment: {}", comment);

    let creation_date = torrent
        .creation_date
        .unwrap_or_else(|| "Optional field \"creation_date\" is empty.".to_owned());
    println!("creation creation_date: {}", creation_date);

    match torrent.http_seeds {
        None => println!("Optional field \"http_seed\" is empty."),
        Some(content) => content
            .iter()
            .for_each(|seed| println!("http seed: {}", seed)),
    }

    println!("piece: {}", torrent.info.file_length);

    Ok(())
}
