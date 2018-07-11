//! An encoder for torrent files
//!
//! This encoder serializes a struct containing torrent information and prints the raw bytes into
//! stdout. Please consider to pipe the output into a file of your choice.
//!
//! You can run this example with:
//!
//!     cargo run --example encode_torrent > example.torrent

extern crate bendy;
extern crate failure;

use std::io::Write;

use failure::Error;

use bendy::{
    encoder::{Encodable, List, SingleItemEncoder},
    Error as BencodeError,
};

/// Main struct containing all torrent information. All fields marked as `not official element`
/// are not part of the torrent but not yet standardized extensions used in torrent file example.
///
/// To keep the example simple we won't parse the bencode integers into actual number types as
/// the bencode integer definition is actually a bigint and the content may nit fit in a specific
/// one.
struct Metainfo {
    pub announce: String,
    pub info: Info,
    pub comment: String,         // not official element
    pub creation_date: String,   // not official element
    pub http_seeds: Vec<String>, // not official element
}

struct Info {
    pub piece_length: String,
    pub pieces: Vec<u8>,
    pub name: String,
    pub file_length: String,
}

impl Encodable for Metainfo {
    const MAX_DEPTH: usize = Info::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"announce", &self.announce)?;
            e.emit_pair(b"comment", &self.comment)?;
            e.emit_pair(b"creation date", &self.creation_date)?;
            e.emit_pair(b"httpseeds", List(&self.http_seeds))?;
            e.emit_pair(b"info", &self.info)
        })
    }
}

impl Encodable for Info {
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"length", &self.file_length)?;
            e.emit_pair(b"name", &self.name)?;
            e.emit_pair(b"piece length", &self.piece_length)?;
            e.emit_pair(b"pieces", &self.pieces)
        })
    }
}

fn main() -> Result<(), Error> {
    let torrent = Metainfo {
        announce: "http://bttracker.debian.org:6969/announce".to_owned(),
        comment: "\"Debian CD from cdimage.debian.org\"".to_owned(),
        creation_date: 1_520_682_848.to_string(),
        http_seeds: vec![
            "https://cdimage.debian.org/cdimage/release/9.4.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-9.4.0-amd64-netinst.iso".to_owned(),
            "https://cdimage.debian.org/cdimage/archive/9.4.0//srv/cdbuilder.debian.org/dst/deb-cd/weekly-builds/amd64/iso-cd/debian-9.4.0-amd64-netinst.iso".to_owned(),
        ],
        info: Info {
            piece_length: 262_144.to_string(),
            pieces: include_bytes!("torrent_files/pieces.iso").to_vec(),
            name: "debian-9.4.0-amd64-netinst.iso".to_owned(),
            file_length: 305_135_616.to_string(),
        },
    };

    let data = torrent.to_bytes()?;
    std::io::stdout().write_all(&data)?;

    Ok(())
}
