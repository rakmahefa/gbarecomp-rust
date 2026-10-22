use std::env;
use std::error::Error;
use std::process::ExitCode;

use gbarecomp::analysis::analyze_rom;
use gbarecomp::rom::RomImage;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Err("missing command".into());
    };
    let Some(path) = args.next() else {
        print_usage();
        return Err("missing ROM path".into());
    };

    let rom = RomImage::load(&path)?;

    match command.as_str() {
        "inspect" => {
            if args.next().is_some() {
                print_usage();
                return Err("unexpected argument for inspect".into());
            }
            println!("ROM: {path}");
            println!("Size: {} bytes ({:.2} MiB)", rom.len(), rom.len() as f64 / 1_048_576.0);
            println!("Title: {}", rom.header.title);
            println!("Game code: {}", rom.header.game_code);
            println!("Maker code: {}", rom.header.maker_code);
            println!("Software version: {}", rom.header.software_version);
            println!(
                "Header checksum: declared=0x{:02X}, computed=0x{:02X} ({})",
                rom.header.declared_checksum,
                rom.header.computed_checksum,
                if rom.header.checksum_matches() { "ok" } else { "mismatch" }
            );
        }
        "analyze" => {
            let max_instructions = match args.next() {
                Some(value) => value.parse::<usize>()?,
                None => 250_000,
            };
            if args.next().is_some() {
                print_usage();
                return Err("too many arguments for analyze".into());
            }
            let report = analyze_rom(&rom, max_instructions);
            println!("Reachable-code analysis (exploratory; not a full recompiler)");
            println!("ROM size: {} bytes", rom.len());
            println!("Blocks: {}", report.blocks.len());
            println!("Decoded instructions: {}", report.instruction_count);
            println!("Unsupported exits: {}", report.unsupported_exit_count());
            println!("Indirect exits: {}", report.indirect_exit_count());
            println!("Instruction limit reached: {}", report.truncated);
            for block in report.blocks {
                println!(
                    "\n0x{:08X} {:?} ({} instructions) -> {:?}",
                    block.start,
                    block.instruction_set,
                    block.instructions.len(),
                    block.exit
                );
                for instruction in block.instructions {
                    println!(
                        "  0x{:08X}: {:0width$X}  {}",
                        instruction.address,
                        instruction.raw,
                        instruction.description,
                        width = usize::from(instruction.width) * 2
                    );
                }
            }
        }
        _ => {
            print_usage();
            return Err(format!("unknown command: {command}").into());
        }
    }
    Ok(())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  gbarecomp inspect <rom.gba>");
    eprintln!("  gbarecomp analyze <rom.gba> [max-instructions]");
}
