use std::io;

use object::{BinaryFormat, Object};

fn unsupported_format<T>(format: impl std::fmt::Display) -> Result<T, Box<dyn std::error::Error>> {
    Err(format!("Unsupported format {:?}", format.to_string()).into())
}

pub trait ObjCopy<'data: 'file, 'file>: Object<'data, 'file> {
    /// format being None indicates raw binary.
    fn write_excluding<W, P>(
        &'file self,
        format: Option<BinaryFormat>,
        writer: W,
        predicate: P,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: std::io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        match format {
            Some(format) => match format {
                BinaryFormat::Coff => self.write_coff(writer, predicate),
                BinaryFormat::Elf => self.write_elf(writer, predicate),
                BinaryFormat::MachO => self.write_macho(writer, predicate),
                BinaryFormat::Pe => self.write_pe(writer, predicate),
                BinaryFormat::Wasm => self.write_wasm(writer, predicate),
                BinaryFormat::Xcoff => self.write_xcoff(writer, predicate),
                format => unsupported_format(format!("{format:?}")),
            },
            None => self.write_raw(writer, predicate),
        }
    }

    fn write_raw<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("raw")
    }

    fn write_coff<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("coff")
    }

    fn write_elf<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("elf")
    }

    fn write_macho<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("macho")
    }

    fn write_pe<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("pe")
    }

    fn write_wasm<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("wasm")
    }

    fn write_xcoff<W, P>(&'file self, _: W, _: P) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        unsupported_format("xcoff")
    }

    fn write_stripped<W>(
        &'file self,
        format: Option<BinaryFormat>,
        writer: W,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: std::io::Write,
    {
        self.write_excluding(format, writer, |s| !self.will_strip(s))
    }

    fn will_strip(&self, _: &Self::Section) -> bool {
        false
    }
}
