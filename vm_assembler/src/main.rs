mod parser;

use clap::{Parser, Subcommand};
use std::error::Error as StdError;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::{fs, os::unix::fs::FileExt};
use vm_cpu::memory::address::Address;
use vm_cpu::registers::Register;
use vm_cpu::{cpu, memory::Memory};

#[derive(Debug, Parser)]
struct Commands {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[command()]
enum Command {
    Compile {
        input: std::path::PathBuf,
        output: std::path::PathBuf,
    },
    Run {
        input: std::path::PathBuf,
    },
    CompileRun {
        input: std::path::PathBuf,
        output: std::path::PathBuf,
    },
}

#[derive(Debug)]
enum Error {
    ParseError(parser::ParseError),
    VmError(vm_cpu::error::Error),
}

impl From<vm_cpu::error::Error> for Error {
    fn from(value: vm_cpu::error::Error) -> Self {
        Self::VmError(value)
    }
}

impl From<vm_cpu::memory::Error> for Error {
    fn from(value: vm_cpu::memory::Error) -> Self {
        Self::VmError(vm_cpu::error::Error::MemError(value))
    }
}

impl From<parser::ParseError> for Error {
    fn from(value: parser::ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl StdError for Error {}

pub struct FileMemory(std::fs::File);

impl Memory for FileMemory {
    fn write<A>(&mut self, address: A, byte: impl Into<u8>) -> Result<(), vm_cpu::memory::Error>
    where
        A: Into<Address> + Copy,
    {
        let offset: usize = usize::from(address.into());
        let byte = byte.into();
        self.0
            .write_at(&[byte], offset as u64)
            .map_err(|_| vm_cpu::memory::Error::InvalidAddress(offset as u16))?;
        Ok(())
    }

    fn read<A>(&self, address: A) -> Result<u8, vm_cpu::memory::Error>
    where
        A: Into<Address> + Copy,
    {
        let offset: usize = usize::from(address.into());
        let mut buff = [0_u8; 1];
        self.0
            .read_exact_at(&mut buff, offset as u64)
            .map_err(|_| vm_cpu::memory::Error::InvalidAddress(offset as u16))?;
        Ok(buff[0])
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("log")
        .expect("truncating log file failed");

    let appender = tracing_appender::rolling::never(".", "log");
    let (appender, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(appender)
        .with_ansi(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);

    let commands = Commands::parse();
    let mut memory = vm_cpu::stack::Stack::<{ u32::MAX as usize }>::new();

    match commands.command {
        Command::Compile { input, output } => {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(output)
                .unwrap();
            let asm_file = fs::read_to_string(&input).unwrap();
            let asm: parser::Parser = asm_file.parse()?;
            let mem = FileMemory(file);
            let mut cpu = cpu::Cpu::new(mem, 0, u32::MAX);
            cpu.write_instructions_to_memory(asm.insts()).unwrap();
        }
        Command::Run { input } => {
            let data = fs::read(&input).unwrap();
            for (i, byte) in data.iter().enumerate() {
                memory
                    .write(i as u16, *byte)
                    .map_err(vm_cpu::error::Error::MemError)?
            }
            let mut cpu = cpu::Cpu::new(memory, 0, u32::MAX);
            cpu.execute();
            println!("executed binary file {input:?}");
            println!(
                "result {:?} {:?}",
                cpu.registers(),
                cpu.memory().get(cpu.registers()[Register::SP], 65535)
            );
        }
        Command::CompileRun { input, output } => {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output)
                .unwrap();
            let asm_file = fs::read_to_string(&input).unwrap();
            let asm: parser::Parser = asm_file.parse()?;

            let mut cpu = cpu::Cpu::new(memory, 0, u32::MAX);
            cpu.write_instructions_to_memory(asm.insts()).unwrap();
            cpu.execute();

            let mem = cpu.memory().get(0, cpu.registers()[Register::IP]).unwrap();
            let _ = file.write(mem);

            println!("executed binary file {input:?}");
            println!(
                "result {:?} {:?}",
                cpu.registers(),
                cpu.memory().get(cpu.registers()[Register::SP], 65535)
            );
        }
    };
    Ok(())
}
