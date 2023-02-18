use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::{Attribute, Cell, Color, Table};
use heck::ToKebabCase;
use objectify::{
    copy::ObjCopy,
    object::{
        read::elf::{ElfFile32, ElfFile64},
        BinaryFormat, Endianness, File, FileKind, Object, ObjectSection, ObjectSegment,
        ObjectSymbol, SectionFlags, SectionKind, SegmentFlags, SymbolScope, SymbolSection,
    },
};

use symbolic_demangle::demangle;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Dump {
            sections,
            segments,
            symbols,
            demangle,
            input,
        } => dump(sections, segments, symbols, demangle, &input),
        Command::Copy {
            input,
            output,
            format,
        } => copy(
            &input,
            output.as_ref().unwrap_or(&input.with_extension(
                input.extension().map_or("copied".to_owned(), |s| {
                    format!("{}.copied", s.to_string_lossy())
                }),
            )),
            format.binary_format(),
        ),
    }
}

fn dump(sections: bool, segments: bool, mut symbols: bool, demangle: bool, input: &Path) {
    let input_bytes = fs::read(input).expect("Failed reading input");
    let object = File::parse(&*input_bytes).expect("Failed parsing object");

    if !sections && !segments && !symbols {
        symbols = true;
    }

    if sections {
        println!("{}", sections_table(&object));
    }

    if symbols {
        println!("{}", symbols_table(&object, demangle))
    }

    if segments {
        println!("{}", segments_table(&object));
    }
}

fn copy(input: &Path, output: &Path, format: Option<BinaryFormat>) {
    let input_bytes = fs::read(input).expect("Failed reading input");
    let mut output_file = fs::File::create(output).expect("Failed opening output file");
    let written_bytes = match FileKind::parse(&*input_bytes).expect("Invalid object kind") {
        FileKind::Elf32 => ElfFile32::<Endianness>::parse(&*input_bytes)
            .expect("Invalid elf32")
            .write_stripped(format, &mut output_file)
            .expect("Failed writing"),
        FileKind::Elf64 => ElfFile64::<Endianness>::parse(&*input_bytes)
            .expect("Invalid elf64")
            .write_stripped(format, &mut output_file)
            .expect("Failed writing"),
        kind => panic!("{kind:?} cannot be copied yet"),
    };

    println!("Wrote {written_bytes} bytes to {:?}", output.display());
}

#[derive(ValueEnum, Debug, Clone)]
enum BinFormat {
    Coff,
    Elf,
    MachO,
    Pe,
    Wasm,
    Xcoff,
    Raw,
}

impl BinFormat {
    fn binary_format(&self) -> Option<BinaryFormat> {
        Some(match self {
            BinFormat::Coff => BinaryFormat::Coff,
            BinFormat::Elf => BinaryFormat::Elf,
            BinFormat::MachO => BinaryFormat::MachO,
            BinFormat::Pe => BinaryFormat::Pe,
            BinFormat::Wasm => BinaryFormat::Wasm,
            BinFormat::Xcoff => BinaryFormat::Xcoff,
            BinFormat::Raw => return None,
        })
    }
}

impl std::fmt::Display for BinFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinFormat::Coff => "coff",
                BinFormat::Elf => "elf",
                BinFormat::MachO => "macho",
                BinFormat::Pe => "pe",
                BinFormat::Wasm => "wasm",
                BinFormat::Xcoff => "xcoff",
                BinFormat::Raw => "raw",
            }
        )
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_version = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /*     /// Verbose output
    #[arg(long, short, default_value_t = false)]
    verbose: bool */
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Dump object file information
    #[command()]
    Dump {
        /// Dump sections
        #[arg(short, long, default_value_t = false)]
        sections: bool,
        /// Dump segments
        #[arg(short = 'g', long, default_value_t = false)]
        segments: bool,
        /// Dump symbols
        #[arg(short = 'm', long, default_value_t = false)]
        symbols: bool,
        /// Demangle symbols
        #[arg(short, long, default_value_t = false)]
        demangle: bool,
        /// Input file
        #[arg()]
        input: PathBuf,
    },

    /// Kinda like objcopy, but worse
    #[command()]
    Copy {
        /// Input file
        #[arg()]
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output binary format
        #[arg(short, long, default_value_t = BinFormat::Raw)]
        format: BinFormat,
    },
}
const COLORS: &[Color] = &[Color::Cyan, Color::Blue];

fn segments_table<'data: 'file, 'file>(object: &'file impl Object<'data, 'file>) -> Table {
    let mut table = Table::new();

    table.set_header(
        [
            "Segment",
            "Memory Region",
            "File Region",
            "Alignment",
            "Flags",
        ]
        .into_iter()
        .map(|s| {
            Cell::new(s)
                .fg(Color::DarkGreen)
                .add_attribute(Attribute::Bold)
        }),
    );

    let mut segments: Vec<_> = object.segments().collect();
    segments.sort_by(|left, right| match left.address().cmp(&right.address()) {
        Ordering::Equal => left.size().cmp(&right.size()),
        ordering => ordering,
    });

    for (index, segment) in segments.iter().enumerate() {
        let color = COLORS[index % COLORS.len()];
        let name = match segment
            .name_bytes()
            .map(|n| n.map(|s| String::from_utf8_lossy(s)))
        {
            Ok(Some(name)) => Cell::new(name).fg(color),
            Ok(None) => Cell::new("none")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Err(_) => Cell::new("error")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
        };

        let memory_region = {
            let start = segment.address();
            let end = start + segment.size();
            Cell::new(format!("{start:#x} -> {end:#x}")).fg(color)
        };

        let file_region = {
            let (start, end) = segment.file_range();
            Cell::new(format!("{start:#x} -> {end:#x}")).fg(color)
        };

        let alignment = Cell::new(format!("{} bytes", segment.align())).fg(color);
        let flags = match segment.flags() {
            SegmentFlags::None => Cell::new("none")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            SegmentFlags::Elf { p_flags: flags }
            | SegmentFlags::Coff {
                characteristics: flags,
            } => Cell::new(format!("{flags:#x}")).fg(color),
            SegmentFlags::MachO {
                flags,
                maxprot,
                initprot,
            } => Cell::new(format!("F({flags:#x}); M({maxprot:#x}); I({initprot:#x})")).fg(color),
            _ => Cell::new("unknown")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        };

        table.add_row([name, memory_region, file_region, alignment, flags]);
    }

    table
}

fn symbols_table<'data: 'file, 'file>(
    object: &'file impl Object<'data, 'file>,
    _demangle: bool,
) -> Table {
    #[allow(unused_mut)]
    let mut table = Table::new();
    table.set_header(
        [
            "Index",
            "Symbol",
            "Memory Region",
            "Kind",
            "Section",
            "Scope",
            // "Flags",
        ]
        .into_iter()
        .map(|s| {
            Cell::new(s)
                .fg(Color::DarkBlue)
                .add_attribute(Attribute::Bold)
        }),
    );

    for symbol in object.symbols() {
        let color = COLORS[symbol.index().0 % COLORS.len()];

        let index = Cell::new(symbol.index().0).fg(color);
        let name = match symbol.name_bytes() {
            Ok(name) => {
                let name = String::from_utf8_lossy(name);
                let name = match _demangle {
                    true => demangle(&name),
                    false => name,
                };

                Cell::new(name).fg(color)
            }
            Err(_) => Cell::new("error")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
        };

        let memory_region = {
            let start = symbol.address();
            let end = start + symbol.size();
            Cell::new(format!("{start:#x} -> {end:#x}")).fg(color)
        };

        let kind = Cell::new(format!("{:?}", symbol.kind()).to_lowercase()).fg(color);
        let section = match symbol.section() {
            SymbolSection::Section(section) => {
                match object.section_by_index(section).and_then(|s| {
                    s.name_bytes()
                        .map(|n| String::from_utf8_lossy(n).to_string())
                }) {
                    Ok(section) => Cell::new(section).fg(color),
                    Err(_) => Cell::new("error")
                        .fg(Color::Red)
                        .add_attribute(Attribute::Bold),
                }
            }
            section => Cell::new(format!("{section:?}").to_lowercase())
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        };

        let scope = match symbol.scope() {
            SymbolScope::Unknown => Cell::new("unknown")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            scope => Cell::new(format!("{scope:?}").to_lowercase()).fg(color),
        };

        table.add_row([index, name, memory_region, kind, section, scope]);
    }

    table
}

fn sections_table<'data: 'file, 'file>(object: &'file impl Object<'data, 'file>) -> Table {
    let mut table = Table::new();
    table.set_header(
        [
            "Index",
            "Section",
            "Memory Region",
            "File Region",
            "Alignment",
            "Kind",
            "Flags",
        ]
        .into_iter()
        .map(|s| {
            Cell::new(s)
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold)
        }),
    );

    for section in object.sections() {
        let color = COLORS[section.index().0 % COLORS.len()];
        let name = match section.name_bytes() {
            Ok(name) => Cell::new(String::from_utf8_lossy(name)).fg(color),
            Err(_) => Cell::new("error")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
        };

        let index = Cell::new(section.index().0).fg(color);
        let memory_region = {
            let start = section.address();
            let end = start + section.size();
            Cell::new(format!("{start:#x} -> {end:#x}")).fg(color)
        };

        let file_region = match section.file_range() {
            Some((start, end)) => Cell::new(format!("{start:#x} -> {end:#x}")).fg(color),
            None => Cell::new("none")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        };

        let alignment = Cell::new(format!("{:#x}", section.align())).fg(color);
        let kind = match section.kind() {
            SectionKind::Elf(elf) => Cell::new(format!("elf({elf:#x})")).fg(color),
            kind => Cell::new(format!("{kind:?}").to_kebab_case()).fg(color),
        };
        let flags = match section.flags() {
            SectionFlags::None => Cell::new("none")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            SectionFlags::Elf { sh_flags } => Cell::new(format!("{sh_flags:#x}")).fg(color),
            SectionFlags::MachO { flags }
            | SectionFlags::Coff {
                characteristics: flags,
            }
            | SectionFlags::Xcoff { s_flags: flags } => Cell::new(format!("{flags:#x}")).fg(color),
            _ => Cell::new("unknown")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        };

        table.add_row([
            index,
            name,
            memory_region,
            file_region,
            alignment,
            kind,
            flags,
        ]);
    }

    table
}
