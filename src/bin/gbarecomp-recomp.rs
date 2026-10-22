//! Build-time ROM analysis and static Rust source emission CLI.
//!
//! The default `gbarecomp-rust` binary starts the Bevy host. Use this second
//! binary for offline, build-time ROM inspection and recompilation.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use gbarecomp_rust::recompiler::analysis::{self, AnalysisOptions};
use gbarecomp_rust::recompiler::codegen::emit_rust_source;
use gbarecomp_rust::rom::{GBA_HEADER_LEN, GbaRom};

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Err("missing command".into());
    };
    let Some(rom_path) = args.next() else {
        print_usage();
        return Err("missing ROM path".into());
    };

    let rom = load_rom(&rom_path)?;
    match command.as_str() {
        "inspect" => {
            if args.next().is_some() {
                print_usage();
                return Err("unexpected argument for inspect".into());
            }
            print_inspection(&rom_path, &rom);
        }
        "analyze" | "recompile" => {
            let mut max_instructions = AnalysisOptions::default().max_instructions;
            let mut output_path: Option<PathBuf> = None;
            while let Some(argument) = args.next() {
                match argument.as_str() {
                    "--max-instructions" => {
                        let raw = args.next().ok_or("--max-instructions needs a value")?;
                        max_instructions = raw.parse()?;
                        if max_instructions == 0 {
                            return Err("--max-instructions must be greater than zero".into());
                        }
                    }
                    "--emit" => {
                        let raw = args.next().ok_or("--emit needs an output path")?;
                        output_path = Some(PathBuf::from(raw));
                    }
                    _ => return Err(format!("unknown option: {argument}").into()),
                }
            }

            if command == "recompile" && output_path.is_none() {
                return Err("recompile requires --emit <generated.rs>".into());
            }

            let report = analysis::analyze(&rom, AnalysisOptions { max_instructions });
            print_analysis(&report);
            if let Some(path) = output_path {
                let source = emit_rust_source(&report)?;
                write_generated_source(&path, &source)?;
                println!("Generated Rust source: {}", path.display());
            } else if command == "recompile" {
                unreachable!("recompile without --emit was rejected above");
            } else if !report.diagnostics.is_empty() {
                println!(
                    "No code was emitted: {} issue(s) require static resolution first.",
                    report.diagnostics.len()
                );
            }
        }
        _ => {
            print_usage();
            return Err(format!("unknown command: {command}").into());
        }
    }
    Ok(())
}

fn load_rom(path: impl AsRef<Path>) -> Result<GbaRom, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    Ok(GbaRom::from_bytes(bytes)?)
}

fn print_inspection(path: &str, rom: &GbaRom) {
    let header = rom.header();
    println!("ROM: {path}");
    println!(
        "Size: {} bytes ({:.2} MiB)",
        rom.len(),
        rom.len() as f64 / 1_048_576.0
    );
    println!("Title: {}", header.title_lossy());
    println!("Game code: {}", String::from_utf8_lossy(&header.game_code));
    println!(
        "Maker code: {}",
        String::from_utf8_lossy(&header.maker_code)
    );
    println!("Entry instruction: 0x{:08X}", header.entry_branch_opcode);
    println!("Fixed value: 0x{:02X}", header.fixed_value);
    println!("Software version: {}", header.software_version);
    println!(
        "Header checksum: declared=0x{:02X}, computed=0x{:02X} ({})",
        header.complement_checksum,
        header.calculated_checksum,
        if header.checksum_valid {
            "ok"
        } else {
            "mismatch"
        }
    );
    println!("Header size: {GBA_HEADER_LEN:#X} bytes");
}

fn print_analysis(report: &gbarecomp_rust::recompiler::ir::AnalysisReport) {
    println!("Static ARM/Thumb discovery (not a compatibility claim)");
    println!(
        "Decoded instruction encodings: {}",
        report.instruction_count
    );
    println!("Basic blocks: {}", report.blocks.len());
    println!("Instruction limit: {}", report.instruction_limit);
    println!("Limit reached: {}", report.reached_instruction_limit);
    println!("Diagnostics: {}", report.diagnostics.len());

    for block in &report.blocks {
        println!(
            "\n0x{:08X} {:?} ({} instruction(s)) -> {:?}",
            block.start,
            block.instruction_set,
            block.instructions.len(),
            block.terminator
        );
        for instruction in &block.instructions {
            println!(
                "  0x{:08X}: {:08X} {:?}",
                instruction.address, instruction.raw, instruction.operation
            );
        }
    }
    for diagnostic in &report.diagnostics {
        println!("DIAGNOSTIC: {diagnostic}");
    }
}

fn write_generated_source(path: &Path, source: &str) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite existing generated source: {}",
            path.display()
        )
        .into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, source)?;
    Ok(())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  gbarecomp-recomp inspect <rom.gba>");
    eprintln!("  gbarecomp-recomp analyze <rom.gba> [--max-instructions N]");
    eprintln!(
        "  gbarecomp-recomp recompile <rom.gba> --emit <generated.rs> [--max-instructions N]"
    );
}
