use std::io;

pub use object;

use object::{
    read::elf::{self, ElfFile},
    ObjectSection, ReadRef, SectionKind,
};

pub trait ObjectExt<'data: 'file, 'file>: object::read::Object<'data, 'file> {
    /// Akin to objcopy -O binary.
    /// I think anyways, that is what I intend.
    fn write_binary<W, P>(
        &'file self,
        writer: W,
        predicate: P,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool;
}

impl<'data: 'file, 'file, T> ObjectExt<'data, 'file> for T
where
    T: object::read::Object<'data, 'file>,
{
    fn write_binary<W, P>(
        &'file self,
        mut writer: W,
        predicate: P,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        let mut sections: Vec<_> = self.sections().filter(predicate).collect();
        sections.sort_by_key(|s| s.address());
        let mut sections = sections.into_iter().peekable();
        let mut written_bytes = 0;

        while let Some(section) = sections.next() {
            let bytes = section.data()?;
            writer.write_all(bytes)?;
            written_bytes += bytes.len();

            if let Some(next_section) = sections.peek() {
                let actual_end_address = section.address() + section.size();
                let expected_end_address = next_section.address();

                if actual_end_address < expected_end_address {
                    let fill_amount = expected_end_address - actual_end_address;
                    written_bytes += fill_amount as usize;

                    for _ in 0..fill_amount {
                        writer.write(&[0x00])?;
                    }
                }
            }
        }

        Ok(written_bytes)
    }
}

pub fn elf_to_bin<'data: 'file, 'file, Elf, R, W>(
    elf: &'file ElfFile<'data, Elf, R>,
    writer: W,
) -> Result<usize, Box<dyn std::error::Error>>
where
    Elf: elf::FileHeader,
    R: ReadRef<'data> + 'file,
    W: io::Write,
{
    elf.write_binary(writer, |s| is_elf_section_alloc(s.kind()))
}

pub fn is_elf_section_alloc(kind: SectionKind) -> bool {
    matches!(
        kind,
        SectionKind::Text
            | SectionKind::Tls
            | SectionKind::Data
            | SectionKind::ReadOnlyString
            | SectionKind::ReadOnlyData
    )
}
