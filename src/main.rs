use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    Succ { target: String, indirect: bool },
    BeqzPred { test: String, test_indirect: bool, jump: String, jump_indirect: bool },
    Exit,
}

impl Instruction {
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        
        if s == "exit" {
            return Some(Instruction::Exit);
        }
        
        let parts: Vec<&str> = s.split_whitespace().collect();
        
        if parts.len() >= 2 && parts[0] == "succ" {
            let target = parts[1];
            if target.starts_with("&") {
                Some(Instruction::Succ {
                    target: target[1..].to_string(),
                    indirect: true,
                })
            } else if target.starts_with("$") {
                Some(Instruction::Succ {
                    target: target[1..].to_string(),
                    indirect: false,
                })
            } else {
                None
            }
        } else if parts.len() >= 3 && parts[0] == "beqz-pred" {
            let test = parts[1];
            let jump = parts[2];
            
            let (test_val, test_indirect) = if test.starts_with("&") {
                (test[1..].to_string(), true)
            } else if test.starts_with("$") {
                (test[1..].to_string(), false)
            } else {
                return None;
            };
            
            let (jump_val, jump_indirect) = if jump.starts_with("&") {
                (jump[1..].to_string(), true)
            } else if jump.starts_with("$") {
                (jump[1..].to_string(), false)
            } else {
                return None;
            };
            
            Some(Instruction::BeqzPred {
                test: test_val,
                test_indirect,
                jump: jump_val,
                jump_indirect,
            })
        } else {
            None
        }
    }
}

struct VM {
    pc: i64,
    memory: Vec<String>,
}

impl VM {
    fn new(pc: i64, memory: Vec<String>) -> Self {
        VM { pc, memory }
    }

    fn get_address(&self, addr_str: &str, indirect: bool) -> i64 {
        let addr = addr_str.parse::<i64>().unwrap_or_else(|_| {
            panic!("Invalid address: {}", addr_str);
        });
        
        if indirect {
            self.check_bounds(addr);
            let value_str = &self.memory[addr as usize];
            value_str.parse::<i64>().unwrap_or_else(|_| {
                panic!("Expected integer at address {} for indirect addressing, found: {}", addr, value_str);
            })
        } else {
            addr
        }
    }

    fn check_bounds(&self, addr: i64) {
        if addr < 0 || addr >= self.memory.len() as i64 {
            panic!("Memory access out of bounds: address {} is beyond memory size {}", 
                   addr, self.memory.len());
        }
    }

    fn execute_instruction(&mut self) -> bool {
        self.check_bounds(self.pc);
        let instruction_str = self.memory[self.pc as usize].clone();
        
        let instruction = Instruction::parse(&instruction_str).unwrap_or_else(|| {
            if instruction_str.parse::<i64>().is_ok() {
                panic!("Trying to execute data value {} at PC={} as instruction", instruction_str, self.pc);
            } else {
                panic!("Invalid instruction at PC={}: {}", self.pc, instruction_str);
            }
        });
        
        println!("PC={}, Executing: {:?}", self.pc, instruction);
        
        match instruction {
            Instruction::Exit => {
                println!("Exit instruction encountered");
                return false;
            }
            Instruction::Succ { target, indirect } => {
                let target_addr = self.get_address(&target, indirect);
                self.check_bounds(target_addr);
                
                let current_val = self.memory[target_addr as usize].parse::<i64>()
                    .unwrap_or(0);
                self.memory[target_addr as usize] = (current_val + 1).to_string();
                self.pc += 1;
            }
            Instruction::BeqzPred { test, test_indirect, jump, jump_indirect } => {
                let test_addr = self.get_address(&test, test_indirect);
                self.check_bounds(test_addr);
                
                let test_val = self.memory[test_addr as usize].parse::<i64>()
                    .unwrap_or(0);
                
                if test_val == 0 {
                    let jump_addr = self.get_address(&jump, jump_indirect);
                    self.check_bounds(jump_addr);
                    self.pc = jump_addr;
                } else {
                    self.memory[test_addr as usize] = (test_val - 1).to_string();
                    self.pc += 1;
                }
            }
        }
        
        true
    }

    fn print_state(&self) {
        println!("\n=== VM State ===");
        println!("PC: {}", self.pc);
        println!("Memory:");
        for (i, val) in self.memory.iter().enumerate() {
            println!("  [{}]: {}", i, val);
        }
        println!("================\n");
    }

    fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            if !self.execute_instruction() {
                self.print_state();
                return;
            }
        }
        self.print_state();
    }
}

fn load_memory_from_file(filename: &str) -> io::Result<Vec<String>> {
    let contents = fs::read_to_string(filename)?;
    let memory: Vec<String> = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();
    Ok(memory)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <initial_pc> <memory_file>", args[0]);
        process::exit(1);
    }
    
    let pc = args[1].parse::<i64>().unwrap_or_else(|_| {
        eprintln!("Invalid PC value: {}", args[1]);
        process::exit(1);
    });
    
    let memory = load_memory_from_file(&args[2]).unwrap_or_else(|e| {
        eprintln!("Failed to load memory file: {}", e);
        process::exit(1);
    });
    
    let mut vm = VM::new(pc, memory);
    
    println!("Turing Machine VM initialized");
    vm.print_state();
    
    loop {
        print!("Enter number of steps to execute (or 'q' to quit): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input == "q" || input == "quit" {
            break;
        }
        
        match input.parse::<usize>() {
            Ok(steps) => {
                if steps == 0 {
                    println!("Please enter a positive number of steps");
                    continue;
                }
                vm.run_steps(steps);
            }
            Err(_) => {
                println!("Invalid input. Please enter a number or 'q' to quit");
            }
        }
    }
}