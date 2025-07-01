use core::fmt::{Display, Write};

use crate::inspect::*;

impl<'a> Display for Inspectable<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Inspectable::String(x) => Display::fmt(x, f),
            Inspectable::Dict(x) => Display::fmt(x, f),
            Inspectable::List(x) => Display::fmt(x, f),
            Inspectable::Int(x) => Display::fmt(x, f),
            Inspectable::Raw(x) => {
                for b in x.iter() {
                    write!(f, "\\x{b:02X}")?;
                }
                Ok(())
            },
        }
    }
}

impl<'a> Display for InString<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = self.content();
        let all_printable = bytes.iter().map(|b| *b).all(is_printable_byte);
        write!(f, "{}:", self.len())?;
        if all_printable {
            for &b in bytes.iter() {
                let c = char::from_u32(b as u32).expect("Already ensured all chars are printable");
                f.write_char(c)?;
            }
            Ok(())
        } else {
            for b in bytes.iter() {
                write!(f, "\\x{b:02X}")?;
            }
            Ok(())
        }
    }
}

impl<'a> Display for InInt<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('i')?;
        f.write_str(&*self.bytes)?;
        f.write_char('e')
    }
}

impl<'a> Display for InDict<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('d')?;
        for InDictEntry { key, value } in self.items.iter() {
            Display::fmt(key, f)?;
            Display::fmt(value, f)?;
        }
        f.write_char('e')
    }
}

impl<'a> Display for InList<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char('l')?;
        for inspectable in self.items.iter() {
            Display::fmt(inspectable, f)?;
        }
        f.write_char('e')
    }
}

impl<'a> Inspectable<'a> {
    pub fn as_rust_string_literal(&self) -> String {
        fn newline(indent: &usize, out: &mut String) {
            out.push_str("\\\n");
            for _ in 0..*indent {
                out.push('\t');
            }
        }

        fn dispatch(inspectable: &Inspectable, indent: &mut usize, out: &mut String) {
            match inspectable {
                x @ Inspectable::Raw(_) | x @ Inspectable::String(_) | x @ Inspectable::Int(_) => {
                    out.push_str(x.to_string().as_str());
                },
                Inspectable::List(InList { items }) => {
                    out.push('l');
                    *indent += 1;
                    for i in items.iter() {
                        newline(indent, out);
                        dispatch(i, indent, out);
                    }
                    *indent -= 1;
                    newline(indent, out);
                    out.push('e');
                },
                Inspectable::Dict(InDict { items }) => {
                    out.push('d');
                    *indent += 1;
                    for i in items.iter() {
                        let InDictEntry { key, value } = i;
                        newline(indent, out);
                        out.push_str(key.to_string().as_str());
                        newline(indent, out);
                        dispatch(value, indent, out);
                    }
                    *indent -= 1;
                    newline(indent, out);
                    out.push('e');
                },
            }
        }

        let mut output = String::new();
        let mut indent = 0;

        output.push_str("let pretty_bencode = b\"\\\n");
        dispatch(self, &mut indent, &mut output);
        output.push_str("\\\n\"");
        output
    }
}

fn is_printable_byte(b: u8) -> bool {
    40 <= b && b < 127
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_printing_works() {
        fn ensure_prints_as(b: impl AsRef<[u8]>, expected: Option<&str>) {
            let b = b.as_ref();
            let i = inspect(b).to_string();
            if let Some(e) = expected {
                assert_eq!(i.as_str(), e);
            } else {
                let e = String::from_utf8(b.to_vec()).unwrap();
                assert_eq!(i, e);
            }
        }
        fn ensure_roundtrip(b: impl AsRef<[u8]>) {
            ensure_prints_as(b, None);
        }
        ensure_roundtrip(b"i200e");
        ensure_roundtrip(b"4:helo");
        ensure_roundtrip(b"de");
        ensure_roundtrip(b"le");
        ensure_roundtrip(b"li2e6:inliste");
        ensure_roundtrip(b"d1:ai1e1:bi2ee");
        ensure_prints_as(
            b"d5:countli1ei2ei3ee4:null1:\x00e",
            Some("d5:countli1ei2ei3ee4:null1:\\x00e"),
        );
    }

    #[ignore]
    #[test]
    fn as_rust_string_literal_test() {
        fn print(b: impl AsRef<[u8]>) {
            let b = b.as_ref();
            let i = inspect(b).as_rust_string_literal();
            println!("-------\n{}", i);
        }
        print(b"i200e");
        print(b"4:helo");
        print(b"de");
        print(b"le");
        print(b"li2e6:inliste");
        print(b"d1:ai1e1:bi2ee");
        print(b"d5:countli1ei2ei3ee4:null1:\x00e");
        let ap = all_printables_bencode_list();
        print(&ap);

        panic!()
    }

    #[ignore]
    #[test]
    fn debug_print_test() {
        let i = inspect(b"i200e");
        dbg!(i);
        let i = inspect(b"4:helo");
        dbg!(i);
        let i = inspect(b"de");
        dbg!(i);
        let i = inspect(b"le");
        dbg!(i);
        let i = inspect(b"li2e4:liste");
        dbg!(i);
        let i = inspect(b"d1:ai1e1:bi2ee");
        dbg!(i);
        panic!()
    }

    #[test]
    fn as_rust_string_literal_doesnt_break_quoting() {
        // We must not considuer double quote symbols printable, since
        // we don't escape them or turn them into \x22 in as_rust_string_literal
        assert!(!is_printable_byte(b'"'));
    }

    fn all_printables_bencode_list() -> Vec<u8> {
        let mut ap = Vec::new();
        ap.push(b'l');
        for b in 40_u8..127 {
            // Just checking that the printable byte ranges are synced
            assert!(is_printable_byte(b));
            ap.push(b'1');
            ap.push(b':');
            ap.push(b);
        }
        ap.push(b'e');
        ap
    }
}
