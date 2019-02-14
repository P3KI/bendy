//! An encoder for torrent files
//!
//! This example will ...
//!
//! - serialize a torrent file representing object in bencode format
//! - and print the result into stdout.
//!
//! *Attention*: Please consider to pipe the output into a file of your choice.
//!
//! # Run the Example
//!
//! ```
//! cargo run --example encode_torrent > example.torrent
//! ```

use std::io::Write;

use bendy::encoding::{AsString, Error as EncodingError, SingleItemEncoder, ToBencode};
use failure::Error;

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

impl ToBencode for MetaInfo {
    // Adds an additional recursion level -- itself formatted as dictionary --
    // around the info struct.
    const MAX_DEPTH: usize = Info::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"announce", &self.announce)?;

            if let Some(comment) = &self.comment {
                e.emit_pair(b"comment", comment)?;
            }

            if let Some(creation_date) = &self.creation_date {
                e.emit_pair(b"creation date", creation_date)?;
            }

            if let Some(seeds) = &self.http_seeds {
                // List is a simple iterable wrapper that allows to encode
                // any list like container as bencode list object.
                e.emit_pair(b"httpseeds", seeds)?;
            }

            e.emit_pair(b"info", &self.info)
        })?;

        Ok(())
    }
}

impl ToBencode for Info {
    // The struct is encoded as dictionary and all of it internals are encoded
    // as flat values, i.e. strings or integers.
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"length", &self.file_length)?;
            e.emit_pair(b"name", &self.name)?;
            e.emit_pair(b"piece length", &self.piece_length)?;
            e.emit_pair(b"pieces", AsString(&self.pieces))
        })?;
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let torrent = MetaInfo {
        announce: "http://bttracker.debian.org:6969/announce".to_owned(),
        comment: Some("\"Debian CD from cdimage.debian.org\"".to_owned()),
        creation_date: Some(1_520_682_848.to_string()),
        http_seeds: Some(vec![
            "https://cdimage.debian.org/cdimage/release/9.4.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-9.4.0-amd64-netinst.iso".to_owned(),
            "https://cdimage.debian.org/cdimage/archive/9.4.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-9.4.0-amd64-netinst.iso".to_owned(),
        ]),
        info: Info {
            piece_length: 262_144.to_string(),
            pieces: include_bytes!("torrent_files/pieces.iso").to_vec(),
            name: "debian-9.4.0-amd64-netinst.iso".to_owned(),
            file_length: 305_135_616.to_string(),
        },
    };

    let data = torrent.to_bencode()?;
    std::io::stdout().write_all(&data)?;

    Ok(())
}
