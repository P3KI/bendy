//! A decoder for torrent files.
//!
//! This example will ...
//!
//! - read a torrent file,
//! - deserialize the bencode formatted information
//! - and print the result into stdout.
//!
//! *Attention*: Please consider to pipe the output into a file of your choice.
//!
//! # Run the Example
//!
//! ```
//! cargo run --example decode_torrent > parsing_output.txt
//! ```

use std::str;

use bendy::{
    decoder::{Decoder, DictDecoder, Object},
    Error as BencodeError,
};
use failure::Fail;

static EXAMPLE_TORRENT: &[u8] =
    include_bytes!("torrent_files/debian-9.4.0-amd64-netinst.iso.torrent");

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
    ///
    /// 1) The expected datatype and
    /// 2) the torrent field for which the data was parsed to give
    ///    the user more data for debugging.
    #[fail(display = "Malformed field. Expected {} in {}.", _0, _1)]
    MalformedField(String, String),
}

impl Error {
    fn unknown_field(field_name: &str) -> Error {
        Error::UnknownField(field_name.into())
    }

    fn missing_field(field_name: &str) -> Error {
        Error::MissingField(field_name.into())
    }

    fn malformed_field(ex_type: &str, field: &str) -> Error {
        Error::MalformedField(ex_type.into(), field.into())
    }
}

impl From<BencodeError> for Error {
    fn from(err: BencodeError) -> Self {
        Error::DecodeError(err)
    }
}

/// Main struct containing all required information.
///
/// Based on: [http://fileformats.wikia.com/wiki/Torrent_file].
///
/// # Design Decision
///
/// To keep the example simple we won't parse the integers fields
/// into a concrete number type as the bencode integer definition
/// is actually a `BigNum` and the content may not fit.
#[derive(Debug)]
struct MetaInfo {
    pub announce: String,
    pub info: Info,
    pub comment: Option<String>,         // not official element
    pub creation_date: Option<String>,   // not official element
    pub http_seeds: Option<Vec<String>>, // not official element
}

/// File related information (Single-file format)
#[derive(Debug)]
struct Info {
    pub piece_length: String,
    pub pieces: Vec<u8>,
    pub name: String,
    pub file_length: String,
}

/// Treats object as bencode integer.
fn decode_integer_as_string(field_name: &str, data: Object) -> Result<String, Error> {
    let error = || Error::malformed_field("Integer", field_name);

    let number_string = data.integer_str_or_else_err(error)?;
    Ok(number_string.to_owned())
}

/// Treats object as byte string.
fn decode_bytes_as_string(field_name: &str, data: Object) -> Result<String, Error> {
    let error = || Error::malformed_field("String", field_name);

    let bytes = data.bytes_or_else_err(error)?;
    let text = str::from_utf8(bytes).map_err(|_| error())?;

    Ok(text.to_owned())
}

/// Treats object as byte string.
fn decode_bytes_as_vec(field_name: &str, data: Object) -> Result<Vec<u8>, Error> {
    let result = data.bytes_or_err(Error::malformed_field("String", field_name))?;
    Ok(result.to_vec())
}

/// Treats object as list of strings.
fn decode_list_of_strings(field_name: &str, data: Object) -> Result<Vec<String>, Error> {
    let mut list_dec = data.list_or_err(Error::malformed_field("List", field_name))?;
    let mut list = Vec::new();

    while let Some(object) = list_dec.next_object()? {
        let list_element = decode_bytes_as_string(field_name, object)?;
        list.push(list_element);
    }

    Ok(list)
}

/// Treats object as dictionary containing all fields for the info struct.
/// On success the dictionary is parsed for the fields of info which are
/// necessary for torrent. Any missing field will result in a missing field
/// error which will stop the decoding.
fn decode_info(field_name: &str, data: Object) -> Result<Info, Error> {
    let mut file_length = None;
    let mut name = None;
    let mut piece_length = None;
    let mut pieces = None;

    let mut dict_dec =
        data.dictionary_or_err(Error::malformed_field("Info Dictionary", field_name))?;

    while let Some(pair) = dict_dec.next_pair()? {
        match pair {
            (b"length", value) => {
                file_length = Some(decode_integer_as_string("torrent.info.file_length", value)?);
            }
            (b"name", value) => {
                name = Some(decode_bytes_as_string("torrent.info.name", value)?);
            }
            (b"piece length", value) => {
                piece_length = Some(decode_integer_as_string("torrent.info.length", value)?);
            }
            (b"pieces", value) => {
                pieces = Some(decode_bytes_as_vec("torrent.info.pieces", value)?);
            }
            (unknown_field, _) => {
                return match str::from_utf8(unknown_field) {
                    Ok(field) => Err(Error::unknown_field(field)),
                    Err(_) => Err(Error::TorrentStructureError),
                }
            }
        }
    }

    let file_length = file_length.ok_or_else(|| Error::missing_field("file_length"))?;
    let name = name.ok_or_else(|| Error::missing_field("name"))?;
    let piece_length = piece_length.ok_or_else(|| Error::missing_field("piece_length"))?;
    let pieces = pieces.ok_or_else(|| Error::missing_field("pieces"))?;

    // Check that we discovered all necessary fields
    Ok(Info {
        file_length,
        name,
        piece_length,
        pieces,
    })
}

/// Entry point for decoding a torrent. The dictionary is parsed for all non-optional and optional fields.
/// Missing optional fields are ignored but any other missing fields result in stopping the decoding and in spawning an `MissingField` error.
fn decode_torrent(mut dict_dec: DictDecoder) -> Result<MetaInfo, Error> {
    let mut announce = None;
    let mut comment = None;
    let mut creation_date = None;
    let mut http_seeds = None;
    let mut info = None;

    while let Some(pair) = dict_dec.next_pair()? {
        match pair {
            (b"announce", value) => {
                announce = Some(decode_bytes_as_string("torrent.announce", value)?);
            }
            (b"comment", value) => {
                comment = Some(decode_bytes_as_string("torrent.comment", value)?);
            }
            (b"creation date", value) => {
                creation_date = Some(decode_integer_as_string("torrent.creation_date", value)?);
            }
            (b"httpseeds", value) => {
                http_seeds = Some(decode_list_of_strings("torrent.http_seeds", value)?);
            }
            (b"info", value) => {
                info = Some(decode_info("torrent.info", value)?);
            }
            (unknown_field, _) => {
                return match str::from_utf8(unknown_field) {
                    Ok(field) => Err(Error::unknown_field(field)),
                    Err(_) => Err(Error::TorrentStructureError),
                };
            }
        }
    }

    Ok(MetaInfo {
        announce: announce.ok_or_else(|| Error::missing_field("announce"))?,
        info: info.ok_or_else(|| Error::missing_field("info"))?,
        comment,
        creation_date,
        http_seeds,
    })
}

fn main() -> Result<(), failure::Error> {
    // Try to parse with a `max_depth` of two.
    //
    // The required max depth of a data structure is calculated as follows:
    //
    //  - Every potential nesting level encoded as bencode dictionary  or list count as +1,
    //  - everything else is ignored.
    //
    // This typically means that we only need to count the amount of nested structs and container
    // types. (Potentially ignoring lists of bytes as they are normally encoded as strings.)
    //
    // struct MetaInfo {                    // encoded as dictionary (+1)
    //    announce: String,
    //    info: Info {                      // encoded as dictionary (+1)
    //      piece_length: String,
    //      pieces: Vec<u8>,                // encoded as string and therefore ignored
    //      name: String,
    //      file_length: String,
    //    },
    //    comment: Option<String>,
    //    creation_date: Option<String>,
    //    http_seeds: Option<Vec<String>>   // if available encoded as list but even then doesn't
    //                                         increase the limit over the deepest chain including
    //                                         info
    // }
    let mut decoder = Decoder::new(EXAMPLE_TORRENT).with_max_depth(2);

    // A bencode encoded torrent file should always start with a base level dictionary.
    let torrent = match decoder.next_object()? {
        Some(Object::Dict(dict)) => decode_torrent(dict)?,
        _ => {
            eprint!("Non-parsable file: Expected bencode dictionary.");
            return Err(Error::TorrentStructureError)?;
        }
    };

    println!("{:#?}", torrent);
    Ok(())
}
