use std::io;

use object::{
    elf::SHF_ALLOC,
    read::elf::{ElfFile, FileHeader},
    Object, ObjectSection, ReadRef, SectionFlags,
};

use crate::copy::ObjCopy;

impl<'data: 'file, 'file, Elf, R> ObjCopy<'data, 'file> for ElfFile<'data, Elf, R>
where
    Elf: FileHeader,
    R: ReadRef<'data> + 'file,
{
    fn write_raw<W, P>(
        &'file self,
        mut writer: W,
        mut predicate: P,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        W: io::Write,
        P: FnMut(&Self::Section) -> bool,
    {
        let mut sections: Vec<_> = self.sections().filter(|s| predicate(s)).collect();
        sections.sort_by_key(|s| s.address());
        let mut written_bytes = 0;
        let mut sections = sections.into_iter().peekable();

        while let Some(section) = sections.next() {
            let bytes = section.data()?;
            writer.write_all(bytes)?;
            written_bytes += bytes.len();

            // TODO: Figure out compression at some point, maybe...
            if let Some(next_section) = sections.peek() {
                let actual_end_address = section.address() + section.size();
                let expected_end_address = next_section.address();

                if actual_end_address < expected_end_address {
                    let fill_amount = expected_end_address - actual_end_address;
                    written_bytes += fill_amount as usize;

                    for _ in 0..fill_amount {
                        writer.write_all(&[0x00])?;
                    }
                }
            }
        }

        Ok(written_bytes)
    }

    fn will_strip(&self, section: &Self::Section) -> bool {
        if let SectionFlags::Elf { sh_flags } = section.flags() {
            // Don't strip alloc sections
            sh_flags & SHF_ALLOC as u64 != SHF_ALLOC as u64
        } else {
            unreachable!()
        }
    }
}

/* fn write_raw<'data, Elf, Sections, Writer, R>(
    writer: Writer,
    sections: Sections,
    elf: &ElfFile<'data, Elf, R>,
) -> Result<usize, Box<dyn std::error::Error>>
where
    Elf: FileHeader,
    Sections: IntoIterator<Item = &'data Elf::SectionHeader>,
    R: ReadRef<'data>,
    Writer: io::Write,
{
    let mut sections: Vec<_> = sections.into_iter().collect();
    sections.sort_by_key(|s| s.sh_addr(elf.endian()).into());
    todo!()
}
 */
