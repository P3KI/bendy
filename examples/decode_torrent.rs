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

use bendy::{
    decoding::{Error, FromBencode, Object, ResultExt},
    encoding::AsString,
};

static EXAMPLE_TORRENT: &[u8] =
    include_bytes!("torrent_files/debian-9.4.0-amd64-netinst.iso.torrent");

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
    pub creation_date: Option<u64>,      // not official element
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

impl FromBencode for MetaInfo {
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
    //    creation_date: Option<u64>,
    //    http_seeds: Option<Vec<String>>   // if available encoded as list but even then doesn't
    //                                         increase the limit over the deepest chain including
    //                                         info
    // }
    const EXPECTED_RECURSION_DEPTH: usize = Info::EXPECTED_RECURSION_DEPTH + 1;

    /// Entry point for decoding a torrent. The dictionary is parsed for all
    /// non-optional and optional fields. Missing optional fields are ignored
    /// but any other missing fields result in stopping the decoding and in
    /// spawning [`DecodingError::MissingField`].
    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut announce = None;
        let mut comment = None;
        let mut creation_date = None;
        let mut http_seeds = None;
        let mut info = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"announce", value) => {
                    announce = String::decode_bencode_object(value)
                        .context("announce")
                        .map(Some)?;
                },
                (b"comment", value) => {
                    comment = String::decode_bencode_object(value)
                        .context("comment")
                        .map(Some)?;
                },
                (b"creation date", value) => {
                    creation_date = u64::decode_bencode_object(value)
                        .context("creation_date")
                        .map(Some)?;
                },
                (b"httpseeds", value) => {
                    http_seeds = Vec::decode_bencode_object(value)
                        .context("http_seeds")
                        .map(Some)?;
                },
                (b"info", value) => {
                    info = Info::decode_bencode_object(value)
                        .context("info")
                        .map(Some)?;
                },
                (unknown_field, _) => {
                    return Err(Error::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                },
            }
        }

        let announce = announce.ok_or_else(|| Error::missing_field("announce"))?;
        let info = info.ok_or_else(|| Error::missing_field("info"))?;

        Ok(MetaInfo {
            announce,
            info,
            comment,
            creation_date,
            http_seeds,
        })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 1;

    /// Treats object as dictionary containing all fields for the info struct.
    /// On success the dictionary is parsed for the fields of info which are
    /// necessary for torrent. Any missing field will result in a missing field
    /// error which will stop the decoding.
    fn decode_bencode_object(object: Object) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut file_length = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"length", value) => {
                    file_length = value
                        .try_into_integer()
                        .context("file.length")
                        .map(ToString::to_string)
                        .map(Some)?;
                },
                (b"name", value) => {
                    name = String::decode_bencode_object(value)
                        .context("name")
                        .map(Some)?;
                },
                (b"piece length", value) => {
                    piece_length = value
                        .try_into_integer()
                        .context("length")
                        .map(ToString::to_string)
                        .map(Some)?;
                },
                (b"pieces", value) => {
                    pieces = AsString::decode_bencode_object(value)
                        .context("pieces")
                        .map(|bytes| Some(bytes.0))?;
                },
                (unknown_field, _) => {
                    return Err(Error::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                },
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
}

fn main() -> Result<(), Error> {
    let torrent = MetaInfo::from_bencode(EXAMPLE_TORRENT)?;
    println!("{:#?}", torrent);
    Ok(())
}
