use std::{env, fs, path::PathBuf};

use obj::elf_to_bin;
use object::{read::elf::ElfFile32, LittleEndian};

fn main() {
    let path = env::args_os().nth(1).expect("Failed to get arg");
    let bytes = fs::read(&path).expect("Failed to read file");
    let elf = ElfFile32::<LittleEndian>::parse(&*bytes).expect("Failed parsing elf");

    let mut file = fs::File::create(format!("{}_obj.bin", PathBuf::from(path).display()))
        .expect("Failed opening file");

    let written_bytes = elf_to_bin(&elf, &mut file).expect("Failed converting elf file");

    println!("Wrote {written_bytes} bytes");
}

/*
pub fn print_segments(data: &[u8]) -> Result<(), object::read::Error> {
    let elf = ElfFile32::<Endianness>::parse(data)?;
    let mut segment_table = Table::new();
    segment_table.set_titles(row![FGb => "Memory Region", "File Region", "Alignment", "Flags"]);
    for segment in elf.segments() {
        let start_address = segment.address();
        let end_address = segment.size() + start_address;
        let (file_start, file_length) = segment.file_range();
        let file_end = file_length + file_start;
        let alignment = segment.align();
        let flags = match segment.flags() {
            SegmentFlags::Elf { p_flags } => p_flags,
            _ => unreachable!(),
        };

        segment_table.add_row(row![
            FB => format!("{start_address:#x} -> {end_address:#x}"),
            format!("{file_start:#x} -> {file_end:#x}"),
            format!("{alignment} bytes"),
            format!("{flags:#x}"),
        ]);
    }
    segment_table.printstd();

    let mut section_table = Table::new();
    section_table.set_titles(row![
        FRb =>
        "Index",
        "Name",
        "Memory Region",
        "File Region",
        "Alignment",
        "Kind",
        "Flags"
    ]);

    for section in &sections {
        let index = section.index().0;
        let name = section.name()?;
        let start_address = section.address();
        let end_address = section.size() + start_address;
        let (file_start, file_length) = section.file_range().unwrap_or((0, 0));
        let file_end = file_length + file_start;
        let alignment = section.align();
        let kind = section.kind();
        let flags = match section.flags() {
            SectionFlags::Elf { sh_flags } => sh_flags,
            _ => unreachable!(),
        };

        section_table.add_row(row![
            FM => format!("{index}"),
            format!("{name}"),
            format!("{start_address:#x} -> {end_address:#x}"),
            format!("{file_start:#x} -> {file_end:#x}"),
            format!("{alignment} bytes"),
            format!("{kind:?}"),
            format!("{flags:#x}"),
        ]);
    }
    section_table.printstd();

    Ok(())
}
 */
