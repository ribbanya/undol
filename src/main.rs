use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};
use config::ConfigError;
use log::{debug, LevelFilter, SetLoggerError, warn};
use object::{Object, ObjectSection, ObjectSymbol, ObjectSymbolTable, Symbol, SymbolIterator};
use object::SymbolKind::Text;
use simple_logger::SimpleLogger;
use crate::MainError::*;
use unwrap_elf::settings::Settings;

enum MainError {
    LoggerError(SetLoggerError),
    SettingsError(ConfigError),
    MissingElfPath,
    MissingSymbolTable,
    ExeHasNoParent,
    FileError(io::Error),
    ElfError(object::Error),
}

type MainResult = Result<(), MainError>;

fn try_init_logger() -> Result<(), SetLoggerError> {
    SimpleLogger::new()
        .with_level(
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Error
            })
        .init()
}

fn try_main() -> MainResult {
    try_init_logger().map_err(LoggerError)?;

    let settings = Settings::new().map_err(SettingsError)?;
    let elf_path = settings.elf.path.ok_or(MissingElfPath)?;
    let data = fs::read(elf_path).map_err(FileError)?;
    let elf = object::File::parse(&*data).map_err(ElfError)?;

    let symbol_table = elf.symbol_table().ok_or(MissingSymbolTable)?;
    let symbols = symbol_table.symbols();

    let out_path = get_out_path()?;
    let mut out_file = fs::File::create(out_path).map_err(FileError)?;

    process_symbols(&elf, symbols, &mut out_file)?;

    Ok(())
}

fn get_out_path() -> Result<PathBuf, MainError> {
    let exe_path = std::env::current_exe().map_err(FileError)?;
    let parent = exe_path.parent().ok_or(ExeHasNoParent)?;
    Ok(parent.join("out.s"))
}

fn process_symbols(elf: &object::File, symbols: SymbolIterator, out: &mut dyn Write) -> MainResult {
    let symbols = symbols.filter(filter_symbol);

    for symbol in symbols {
        process_symbol(symbol, elf, out)?;
    }

    Ok(())
}

fn filter_symbol(symbol: &Symbol) -> bool {
    if !symbol.is_definition() { return false; }
    if symbol.kind() != Text { return false; };

    if symbol.size() == 0 {
        debug!("'{}' has no size", symbol.name().unwrap_or(&symbol.address().to_string()));
        return false;
    }

    return true;
}

fn process_symbol(symbol: Symbol, elf: &object::File, out: &mut dyn Write) -> MainResult {
    let address64 = symbol.address();

    let address32: u32 = {
        let result: Result<u32, _> = address64.try_into();
        match result {
            Ok(value32) => value32,
            Err(err) => {
                warn!("Couldn't convert {address64:X} to u32 ({err})");
                return Ok(());
            }
        }
    };

    let index = match symbol.section_index() {
        Some(value) => value,
        None => {
            warn!("Couldn't get section index for symbol @{address32:08X}");
            return Ok(());
        }
    };

    let section = elf.section_by_index(index).map_err(ElfError)?;
    let range = section.data_range(address64, symbol.size());

//             section
//                 .and_then(|c| c.data_range(address as u64, sym.size()).ok())
//                 .flatten()
//                 .map(|c| disasm_iter(c, address))
//                 .map(|disasm| (sym, disasm))
//         })
//         .for_each(|(sym, disasm)| {
//             if let Ok(name) = sym.name() {
//                 write!(out_file, "{name}:\n").unwrap();
//                 for ins in disasm {
//                     let code = ins.code;
//                     let address = ins.addr;
//                     let simplified = ins.simplified();
//                     let simplified_str = simplified.to_string();
//                     write!(out_file, "/* {address:08X} {code:08X} */ {simplified_str}\n")// .unwrap();
//                 }
//             }
//         });

    writeln!(out, "{range:?}").map_err(FileError)?;

    Ok(())
}

fn handle_error(error: MainError) {
    match error {
        LoggerError(inner) => todo!("{}", inner.to_string()),
        SettingsError(inner) => todo!("{inner}"),
        MissingElfPath => todo!("Missing elf path"),
        MissingSymbolTable => todo!("Missing symbol table"),
        FileError(inner) => todo!("{inner}"),
        ElfError(inner) => todo!("{inner}"),
        ExeHasNoParent => todo!("This shouldn't happen...")
    };
}

fn main() {
    let before = Instant::now();

    if let Err(error) = try_main() {
        handle_error(error);
    }

    println!("\nElapsed time: {:.2?}", before.elapsed());
//
}
