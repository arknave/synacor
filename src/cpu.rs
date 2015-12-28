use std::char;
use std::io;

pub const MEM_SIZE: usize = 1 << 16;
const REG_SIZE: usize = 8;
const MAX_VAL: u16 = 32768;

static INSTRUCTION_TABLE: [(fn(&mut Cpu, Vec<u16>)->(), usize); 22] = [
    (Cpu::halt, 0), // 0
    (Cpu::set, 2), // 1
    (Cpu::push, 1), // 2
    (Cpu::pop, 1), // 3
    (Cpu::eq, 3), // 4
    (Cpu::gt, 3), // 5
    (Cpu::jmp, 1), // 6
    (Cpu::jt, 2), // 7
    (Cpu::jf, 2), // 8
    (Cpu::add, 3), // 9
    (Cpu::mult, 3), // 10
    (Cpu::_mod, 3), // 11
    (Cpu::and, 3), // 12
    (Cpu::or, 3), // 13
    (Cpu::not, 2), // 14
    (Cpu::rmem, 2), // 15
    (Cpu::wmem, 2), // 16
    (Cpu::call, 1), // 17
    (Cpu::ret, 0), // 18
    (Cpu::out, 1), // 19
    (Cpu::_in, 1), // 20
    (Cpu::noop, 0) // 21
];

pub struct Cpu {
    memory: [u16; MEM_SIZE],
    registers: [u16; REG_SIZE],
    stack: Vec<u16>,
    pc: usize,
    pub enabled: bool,
    input_buffer: Vec<u8>,
    input_index: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            memory: [0; MEM_SIZE],
            registers: [0; REG_SIZE],
            stack: vec![],
            pc: 0,
            enabled: true,
            input_buffer: vec![],
            input_index: 0
        }
    }

    /*
     * Assorted utility functions
     */
    pub fn load_memory(&mut self, mem: &mut [u16; MEM_SIZE]) {
        self.memory = *mem;
    }

    pub fn print_memory(&self) {
        for (index, value) in self.memory.iter().enumerate()  {
            println!("[{:05}]: {:05}", index, value);
        }
    }

    fn reg_lit(&self, val: u16) -> u16 {
        if val < MAX_VAL {
            val
        } else {
            self.registers[(val % MAX_VAL) as usize]
        }
    }

    fn bin_op<F>(&mut self, args: Vec<u16>, op: F) 
            where F: Fn(u16, u16) -> u16 {

        let reg = self.reg_index(args[0]);
        let a = self.reg_lit(args[1]);
        let b = self.reg_lit(args[2]);

        self.registers[reg as usize] = op(a, b) % MAX_VAL;
    }

    #[inline]
    fn reg_index(&self, mem: u16) -> u16 {
        if MAX_VAL <= mem && mem < MAX_VAL + 8 {
            return mem % MAX_VAL
        } else {
            panic!("Invalid register index {}", mem);
        }
    }

    /*
     * Actual CPU Instructions below
     */

    fn halt(&mut self, _: Vec<u16>) {
        println!("Halted with opcode {}", self.memory[self.pc - 1]);
        self.enabled = false;
    }

    fn push(&mut self, args: Vec<u16>) {
        let val = self.reg_lit(args[0]);
        self.stack.push(val);
    }

    fn pop(&mut self, args: Vec<u16>) {
        let reg = self.reg_index(args[0]);
        let val = self.stack.pop().expect("Empty stack");
        self.registers[reg as usize] = val;
    }

    fn set(&mut self, args: Vec<u16>) {
        let reg = self.reg_index(args[0]);
        let val = self.reg_lit(args[1]);

        self.registers[reg as usize] = val;
    }

    fn eq(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| (a == b) as u16);
    }

    fn gt(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| (a > b) as u16);
    }

    fn jmp(&mut self, args: Vec<u16>) {
        let new_pc = self.reg_lit(args[0]);
        self.pc = self.reg_lit(new_pc) as usize;
    }

    fn jt(&mut self, args: Vec<u16>) {
        let nonzero = self.reg_lit(args[0]);
        let new_pc = self.reg_lit(args[1]);

        if nonzero != 0 {
            self.pc = new_pc as usize;
        }
    }

    fn jf(&mut self, args: Vec<u16>) {
        let zero = self.reg_lit(args[0]);
        let new_pc = self.reg_lit(args[1]);

        if zero == 0 {
            self.pc = new_pc as usize;
        }
    }

    // This can never overflow outside a u16
    fn add(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| a + b);
    }

    // The conversion to u32 needs to be done in order to handle overflow
    fn mult(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| ((a as u32) * (b as u32) % (MAX_VAL as u32)) as u16);
    }

    // Called _mod to avoid reserved word conflict with Rust's `mod`
    fn _mod(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| a % b);
    }

    fn and(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| a & b);
    }

    fn or(&mut self, args: Vec<u16>) {
        self.bin_op(args, |a, b| a | b);
    }

    fn not(&mut self, args: Vec<u16>) {
        let reg = self.reg_index(args[0]);
        let a = self.reg_lit(args[1]);

        self.registers[reg as usize] = (!a) & 0x7fff
    }

    fn rmem(&mut self, args: Vec<u16>) {
        let reg = self.reg_index(args[0]);

        let ind = self.reg_lit(args[1]);
        let val = self.memory[ind as usize];

        self.registers[reg as usize] = val;
    }

    fn wmem(&mut self, args: Vec<u16>) {
        let addr = self.reg_lit(args[0]);
        let val = self.reg_lit(args[1]);

        self.memory[addr as usize] = val;
    }

    fn out(&mut self, args: Vec<u16>) {
        let ascii = self.reg_lit(args[0]) as u32;
        print!("{}", char::from_u32(ascii).expect("Invalid code"));
    }

    // Called _in to avoid reserved word conflict with Rust's `in`
    fn _in(&mut self, args: Vec<u16>) {
        let reg = self.reg_index(args[0]) as usize;

        if self.input_index < self.input_buffer.len() {
            self.registers[reg] = self.input_buffer[self.input_index] as u16;
            self.input_index += 1;
        } else {
            let mut string = String::new();
            io::stdin().read_line(&mut string).unwrap();
            self.input_buffer = string.into_bytes();

            self.registers[reg] = self.input_buffer[0] as u16;
            self.input_index = 1;
        }
    }

    fn call(&mut self, args: Vec<u16>) {
        self.stack.push(self.pc as u16);
        self.jmp(args);
    }

    fn ret(&mut self, _: Vec<u16>) {
        let top_opt = self.stack.pop();

        if top_opt.is_none() {
            return self.halt(vec![]);
        }

        let addr = top_opt.expect("Impossible");

        let args = vec![addr];
        self.jmp(args);
    }

    fn noop(&mut self, _: Vec<u16>) {
    }

    /*
     * Steps through a single execution cycle. Use in a loop
     * to run the whole program.
     */
    pub fn execute(&mut self) {
        let opcode = self.memory[self.pc];
        if opcode > 21 {
            panic!("Invalid opcode {}", opcode);
        }
        //println!("{}", opcode);
        let (f, arg_len) = INSTRUCTION_TABLE[opcode as usize];
        let args = self.memory[self.pc + 1 .. self.pc + arg_len + 1].to_vec();
        self.pc += 1 + arg_len;
        f(self, args)
    }
}
