use std::fmt::{self};

type InstructionFn = fn(&mut Interpreter, addr: usize);

pub struct Interpreter {
    pub acc: u8,
    pub zero_f: bool,
    pub negative_f: bool,
    pub mem: [u8; 256],
    pub pc: u8,
    pub should_stop: bool,
}

fn nop(_i: &mut Interpreter, _addr: usize) {
    // println!("função nop chamada");
}

fn sta(i: &mut Interpreter, addr: usize) {
    i.mem[addr] = i.acc;
}

fn lda(i: &mut Interpreter, addr: usize) {
    i.acc = i.mem[addr];
}

fn add(i: &mut Interpreter, addr: usize) {
    i.acc += i.mem[addr];
}

fn or(i: &mut Interpreter, addr: usize) {
    i.acc |= i.mem[addr];
}

fn and(i: &mut Interpreter, addr: usize) {
    i.acc &= i.mem[addr];
}

fn not(i: &mut Interpreter, _addr: usize) {
    i.acc = !i.acc;
}

fn jmp(i: &mut Interpreter, addr: usize) {
    i.pc = addr as u8;
}

fn jn(i: &mut Interpreter, addr: usize) {
    if i.negative_f {
        jmp(i, addr);
    }
}

fn jz(i: &mut Interpreter, addr: usize) {
    if i.zero_f {
        jmp(i, addr);
    }
}

fn hlt(i: &mut Interpreter, _addr: usize) {
    i.should_stop = true;
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            acc: 0,
            pc: 0,
            zero_f: true,
            negative_f: false,
            mem: [0; 256],
            should_stop: false,
        }
    }

    pub fn load_neander_file(mut self, file_data: &[u8]) -> Result<Self, String> {
        if file_data.len() != 516 {
            return Err("Invalid file. The length of file wasn't 516 bytes.".to_string());
        }
        if file_data[0..4] != [3, 78, 68, 82] {
            return Err("Invalid header format.".to_string());
        }

        for i in 0..256 {
            self.mem[i] = file_data[4 + (i * 2)];
        }

        Ok(self)
    }

    pub fn set_pc(mut self, pc: u8) -> Self {
        self.pc = pc;
        self
    }

    fn get_rules(opcode: u8) -> Option<InstructionFn> {
        match opcode {
            0 => Some(nop),
            16 => Some(sta),
            32 => Some(lda),
            48 => Some(add),
            64 => Some(or),
            80 => Some(and),
            96 => Some(not),
            128 => Some(jmp),
            144 => Some(jn),
            160 => Some(jz),
            240 => Some(hlt),
            _ => None,
        }
    }

    fn fetch(&mut self) -> u8 {
        let code = self.mem[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        code
    }

    pub fn run(&mut self) {
        while (self.pc as usize) < self.mem.len() && !self.should_stop {
            let opcode = self.fetch();

            if let Some(function) = Interpreter::get_rules(opcode) {
                if opcode == 0 || opcode == 96 || opcode == 240 {
                    function(self, 0);
                    continue;
                }

                let addr = self.fetch() as usize;
                function(self, addr);
            } else {
                continue;
            }
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Interpreter: \n ACC: {}\n PC: {}\n", self.acc, self.pc)?;

        for chunk in self.mem.chunks(16) {
            for byte in chunk {
                write!(f, " {:02}", byte)?;
            }
            writeln!(f)?;
        }

        writeln!(f)
    }
}
