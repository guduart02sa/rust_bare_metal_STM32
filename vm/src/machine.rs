use std::io::{self, Write};

pub const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Debug)]
pub enum Error {
    MemoryOverflow,
    WrongInstruction(u8),
    NoReg(usize),
    NoMem(u32),
    OutputError,
    NonExistingInstruction,
    NonExistingFormat,
    ReadPastMemoryEnd,
}

pub struct Machine {
    memory: [u8; MEMORY_SIZE],
    regs: [u32; NREGS],
}

impl Machine {
    /// Creates a new machine in its reset state with memory initialized from input
    pub fn new(memory: &[u8]) -> Result<Self> {
        if memory.len() > MEMORY_SIZE {
            return Err(Error::MemoryOverflow);
        }
        let mut machine = Self {
            memory: [0; MEMORY_SIZE],
            regs: [0; NREGS],
        };
        machine.memory[..memory.len()].copy_from_slice(memory);
        Ok(machine)
    }

    /// Runs the machine until the program terminates or an error occurs
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<()> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Convenience function for running the machine with stdout
    pub fn run(&mut self) -> Result<()> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Executes the next instruction, returning true if the program should terminate
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        if ip >= MEMORY_SIZE {
            return Err(Error::NoMem(ip as u32));
        }

        let opcode = self.memory[ip];
        match opcode {
            1 => self.move_if(fd),
            2 => self.store(),
            3 => self.load(),
            4 => self.load_imm(),
            5 => self.sub(),
            6 => self.out(fd),
            7 => {        self.inc(1);
                return Ok(true);}
            8 => self.out_number(fd),
            _ => Err(Error::WrongInstruction(opcode)),
        }
    }

    fn move_if<T: Write>(&mut self, _fd: &T) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg_a = self.memory[ip + 1] as usize;
        let reg_b = self.memory[ip + 2] as usize;
        let reg_c = self.memory[ip + 3] as usize;

        if reg_a >= NREGS || reg_b >= NREGS || reg_c >= NREGS {
            return Err(Error::NoReg(reg_a.max(reg_b.max(reg_c))));
        }

        if self.regs[reg_c] != 0 {
            self.regs[reg_a] = self.regs[reg_b];
        }

        self.regs[IP] += 4;
        Ok(false)
    }

    fn store(&mut self) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg_a = self.memory[ip + 1] as usize;
        let reg_b = self.memory[ip + 2] as usize;

        if reg_a >= NREGS || reg_b >= NREGS {
            return Err(Error::NoReg(reg_a.max(reg_b)));
        }

        let addr = self.regs[reg_a] as usize;
        if addr >= MEMORY_SIZE {
            return Err(Error::NoMem(self.regs[reg_a]));
        }

        let bytes = self.regs[reg_b].to_le_bytes();
        self.memory[addr..addr+4].copy_from_slice(&bytes);

        self.regs[IP] += 3;
        Ok(false)
    }

    fn load(&mut self) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg_a = self.memory[ip + 1] as usize;
        let reg_b = self.memory[ip + 2] as usize;

        if reg_a >= NREGS || reg_b >= NREGS {
            return Err(Error::NoReg(reg_a.max(reg_b)));
        }

        let addr = self.regs[reg_b] as usize;
        if addr >= MEMORY_SIZE {
            return Err(Error::NoMem(self.regs[reg_b]));
        }

        let value = u32::from_le_bytes([
            self.memory[addr],
            self.memory[addr + 1],
            self.memory[addr + 2],
            self.memory[addr + 3],
        ]);

        self.regs[reg_a] = value;

        self.regs[IP] += 3;
        Ok(false)
    }

    fn load_imm(&mut self) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg_a = self.memory[ip + 1] as usize;
        let l = self.memory[ip + 2];
        let h = self.memory[ip + 3];

        if reg_a >= NREGS {
            return Err(Error::NoReg(reg_a));
        }

        let value = i32::from(i16::from_le_bytes([l, h])) as u32;
        self.regs[reg_a] = value;

        self.regs[IP] += 4;
        Ok(false)
    }

    fn sub(&mut self) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg_a = self.memory[ip + 1] as usize;
        let reg_b = self.memory[ip + 2] as usize;
        let reg_c = self.memory[ip + 3] as usize;

        if reg_a >= NREGS || reg_b >= NREGS || reg_c >= NREGS {
            return Err(Error::NoReg(reg_a.max(reg_b.max(reg_c))));
        }

        self.regs[reg_a] = self.regs[reg_b].wrapping_sub(self.regs[reg_c]);

        self.regs[IP] += 4;
        Ok(false)
    }

    pub fn inc(&mut self, offset: u32) -> () {
        self.regs[IP] += offset;
    }

    fn out<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg = self.memory[ip + 1] as usize;

        if reg >= NREGS {
            return Err(Error::NoReg(reg));
        }

        let character = self.regs[reg] as u8 as char;
        write!(fd, "{}", character).map_err(|_| Error::OutputError)?;

        self.regs[IP] += 2;
        Ok(false)
    }

    fn out_number<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        let ip = self.regs[IP] as usize;
        let reg = self.memory[ip + 1] as usize;

        if reg >= NREGS {
            return Err(Error::NoReg(reg));
        }

        let number = self.regs[reg] as i32;
        write!(fd, "{}", number).map_err(|_| Error::OutputError)?;

        self.regs[IP] += 2;
        Ok(false)
    }

     /// Similar to [`step_on`](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    #[must_use]
    pub fn regs(&self) -> &[u32] {
        &self.regs
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<()> {
        if reg >= NREGS {
            return Err(Error::NoReg(reg));
        }
        self.regs[reg] = value;
        Ok(())
    }

    /// Reference onto the machine current memory.
    #[must_use]
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    
}
