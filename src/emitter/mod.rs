use std::{
    collections::{HashMap, HashSet, VecDeque}, fmt::Display
};
use error::CompilerError;
use instruct::{Instruction, ProcedureBuilder};

pub mod error;
pub mod instruct;
use crate::ast::*;

use Registers::*;




#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Registers {
    A,B,C,D,E,F,G,H,
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            A => write!(f, "a"),
            B => write!(f, "b"),
            C => write!(f, "c"),
            D => write!(f, "d"),
            E => write!(f, "e"),
            F => write!(f, "f"),
            G => write!(f, "g"),
            H => write!(f, "h"),
        }
    }
}

#[derive(Debug)]
enum VariableVariant {
    Atomic(u64),
    Table(u64, u64),
}

#[derive(Debug)]
pub struct Emitter {
    pseudo_assembly: Vec<Instruction>,
    procedures: HashMap<String, ProcedureBuilder>,
    memory: HashMap<String, VariableVariant>,
    initialisated_variables: HashSet<String>,
    memory_pointer: u64,
    ast: Program,
}
fn put_in_a(mut num: u64) -> Vec<Instruction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    instructions.push(Instruction::Rst(A));
    if num != 0 {
        instructions.push(Instruction::Inc(A));
        let mut sub_instructions: Vec<Instruction> = Vec::new();
        while num != 1 {
            if num % 2 == 1 {
                sub_instructions.push(Instruction::Inc(A));
                num -= 1;
            } else {
                sub_instructions.push(Instruction::Shl(A));
                num /= 2;
            }
        }
        sub_instructions.reverse();
        instructions.extend(sub_instructions);
    }
    instructions
}

impl Emitter {
    pub fn new(ast: Program) -> Result<Emitter, CompilerError> {
        let mut procedures: HashMap<String, ProcedureBuilder> = HashMap::new();
        if let Some(procedures_ast) = ast.0.clone() {
            for procedure in procedures_ast {
                if procedures.insert(procedure.0.0.0.clone(), ProcedureBuilder::new(procedure.clone())).is_some() {
                    Err(CompilerError::DuplicateProcedureDeclaration(procedure.0.0.0.clone(), procedure.0.0.1.clone()))?;
                }
            }
        }
        let mut memory_pointer: u64 = 0;
        let mut memory: HashMap<String, VariableVariant> = HashMap::new();
        if let Some(vars) = ast.1 .0.clone() {
            for var in vars {
                match var {
                    DeclarationVariant::Base(id) => {
                        memory.insert(id.0, VariableVariant::Atomic(memory_pointer));
                        memory_pointer += 1;
                    }
                    DeclarationVariant::NumIndexed(id, size) => {
                        memory.insert(id.0, VariableVariant::Table(memory_pointer, size));
                        memory_pointer += size;
                    }
                }
            }
        }
        Ok(Emitter {
            pseudo_assembly: vec![],
            procedures,
            memory,
            memory_pointer,
            ast,
            initialisated_variables: HashSet::new(),
        })
    }
    pub fn emit(&self) -> String {
        let mut assembly: Vec<String> = Vec::new();
        for instruction in &self.pseudo_assembly {
            match instruction {
                Instruction::Read => assembly.push("READ\n".to_string()),
                Instruction::Write => assembly.push("WRITE\n".to_string()),
                Instruction::Load(register) => assembly.push(format!("LOAD {}\n", register)),
                Instruction::Store(register) => assembly.push(format!("STORE {}\n", register)),
                Instruction::Add(register) => assembly.push(format!("ADD {}\n", register)),
                Instruction::Sub(register) => assembly.push(format!("SUB {}\n", register)),
                Instruction::Get(register) => assembly.push(format!("GET {}\n", register)),
                Instruction::Put(register) => assembly.push(format!("PUT {}\n", register)),
                Instruction::Rst(register) => assembly.push(format!("RST {}\n", register)),
                Instruction::Inc(register) => assembly.push(format!("INC {}\n", register)),
                Instruction::Dec(register) => assembly.push(format!("DEC {}\n", register)),
                Instruction::Shl(register) => assembly.push(format!("SHL {}\n", register)),
                Instruction::Shr(register) => assembly.push(format!("SHR {}\n", register)),
                Instruction::Jump(offset) => {
                    assembly.push(format!("JUMP {}\n", offset + assembly.len() as i64))
                }
                Instruction::Jpos(offset) => {
                    assembly.push(format!("JPOS {}\n", offset + assembly.len() as i64))
                }
                Instruction::Jzero(offset) => {
                    assembly.push(format!("JZERO {}\n", offset + assembly.len() as i64))
                }
                Instruction::Strk => {
                    assembly.push(format!("PUT k\n")); // rx ← k
                    assembly.push("INC k\n".to_string()); // k ← k + 1
                }
                Instruction::Jumpr(register) => {
                    assembly.push(format!("GET {}\n", register)); // k ← rx
                }
                Instruction::Halt => assembly.push("HALT\n".to_string()),
                Instruction::Mul => {
                    assembly.push("PUT e\n".to_string()); // 0 1
                    assembly.push("ADD e\n".to_string()); //2
                    assembly.push("SUB e\n".to_string()); //3
                    assembly.push("RST f\n".to_string()); //4
                    assembly.push("GET c\n".to_string()); // 5
                    assembly.push(format!("JZERO {}\n", assembly.len() + 14)); // 6
                    assembly.push("SHR e\n".to_string());
                    assembly.push("SHL e\n".to_string());
                    assembly.push("GET c\n".to_string());
                    assembly.push("SUB e\n".to_string());
                    assembly.push(format!("JZERO {}\n", assembly.len() + 4)); // 11
                    assembly.push("GET f\n".to_string());
                    assembly.push("ADD b\n".to_string());
                    assembly.push("PUT f\n".to_string());
                    assembly.push("SHL b\n".to_string());
                    assembly.push("SHR c\n".to_string());
                    assembly.push("GET c\n".to_string());
                    assembly.push("PUT e\n".to_string()); //18
                    assembly.push(format!("JPOS {}\n", assembly.len() - 14)); // 19
                    assembly.push("GET f\n".to_string()); // 20
                }
                Instruction::Div => {
                    assembly.push("RST d\n".to_string()); // 0 1
                    assembly.push("ADD e\n".to_string());
                    assembly.push("SUB e\n".to_string());
                    assembly.push(format!("JZERO {}\n", assembly.len() + 21)); // 1 2
                    assembly.push("GET c\n".to_string()); // 2 3
                    assembly.push("SUB b\n".to_string());
                    assembly.push(format!("JPOS {}\n", assembly.len() + 18)); // 4 5
                    assembly.push("GET c\n".to_string());
                    assembly.push("PUT e\n".to_string());
                    assembly.push("RST f\n".to_string());
                    assembly.push("INC f\n".to_string());
                    assembly.push("GET e\n".to_string()); // 9 10
                    assembly.push("SUB b\n".to_string());
                    assembly.push(format!("JPOS {}\n", assembly.len() + 10)); // 11 12
                    assembly.push("GET b\n".to_string());
                    assembly.push("SUB e\n".to_string());
                    assembly.push("PUT b\n".to_string());
                    assembly.push("GET d\n".to_string());
                    assembly.push("ADD f\n".to_string());
                    assembly.push("PUT d\n".to_string());
                    assembly.push("SHL e\n".to_string());
                    assembly.push("SHL f\n".to_string());
                    assembly.push(format!("JPOS {}\n", assembly.len() - 11)); // 20 21
                    assembly.push(format!("JPOS {}\n", assembly.len() - 19)); // 21 22
                    assembly.push("GET d\n".to_string()); // 22 23
                }
                Instruction::Mod => {
                    assembly.push("RST d\n".to_string()); // 0 1
                    assembly.push("ADD e\n".to_string());
                    assembly.push("SUB e\n".to_string());
                    assembly.push(format!("JZERO {}\n", assembly.len() + 21)); // 1 2
                    assembly.push("GET c\n".to_string()); // 2 3
                    assembly.push("SUB b\n".to_string());
                    assembly.push(format!("JPOS {}\n", assembly.len() + 19)); // 4 5
                    assembly.push("GET c\n".to_string());
                    assembly.push("PUT e\n".to_string());
                    assembly.push("RST f\n".to_string());
                    assembly.push("INC f\n".to_string());
                    assembly.push("GET e\n".to_string()); // 9 10
                    assembly.push("SUB b\n".to_string());
                    assembly.push(format!("JPOS {}\n", assembly.len() + 10)); // 11 12
                    assembly.push("GET b\n".to_string());
                    assembly.push("SUB e\n".to_string());
                    assembly.push("PUT b\n".to_string());
                    assembly.push("GET d\n".to_string());
                    assembly.push("ADD f\n".to_string());
                    assembly.push("PUT d\n".to_string());
                    assembly.push("SHL e\n".to_string());
                    assembly.push("SHL f\n".to_string());
                    assembly.push(format!("JUMP {}\n", assembly.len() - 11)); // 20 21
                    assembly.push(format!("JUMP {}\n", assembly.len() - 19)); // 21 22
                    assembly.push("RST b\n".to_string()); // 22 23
                    assembly.push("GET b\n".to_string()); // 23 24
                }
            }
        }

        let mut assembled = String::new();
        for line in assembly {
            assembled += &line;
        }
        assembled
    }    pub fn construct(&mut self) -> Result<(), CompilerError>{
        self.construct_main()?;
        self.pseudo_assembly.push(Instruction::Halt);
        Ok(())
    }

    /// Constructs a sequence of instructions for evaluating an expression.
    /// Supports various operations (Add, Sub, Mul, Div, Mod) by generating
    /// the appropriate instruction sequences for each.
    fn make_expressions(&self, expression: Expression) -> Result<Vec<Instruction>, CompilerError> {
        match expression {
            Expression::Value(value) => {
                self.check_if_initialised(value.clone());
                self.extract_value(value)
            },
            Expression::Add(value_0, value_1) => {
                self.check_if_initialised(value_0.clone());
                let mut instructions = self.extract_value(value_0)?;
                instructions.push(Instruction::Put(B));
                self.check_if_initialised(value_1.clone());
                instructions.extend(self.extract_value(value_1)?);
                instructions.push(Instruction::Add(B));
                Ok(instructions)
            }
            Expression::Sub(value_0, value_1) => {
                self.check_if_initialised(value_1.clone());
                let mut instructions = self.extract_value(value_1)?;
                instructions.push(Instruction::Put(B));
                self.check_if_initialised(value_0.clone());
                instructions.extend(self.extract_value(value_0)?);
                instructions.push(Instruction::Sub(B));
                Ok(instructions)
            }
            Expression::Mul(value_0, value_1) => {
                self.check_if_initialised(value_0.clone());
                let mut instructions = self.extract_value(value_0)?;
                instructions.push(Instruction::Put(B));
                self.check_if_initialised(value_1.clone());
                instructions.extend(self.extract_value(value_1)?);
                instructions.push(Instruction::Put(C));
                instructions.push(Instruction::Mul);
                Ok(instructions)
            }
            Expression::Div(value_0, value_1) => {
                self.check_if_initialised(value_0.clone());
                self.check_if_initialised(value_1.clone());
                let mut instructions = self.extract_value(value_0)?;
                instructions.push(Instruction::Put(B));
                instructions.extend(self.extract_value(value_1)?);
                instructions.push(Instruction::Put(C));
                instructions.push(Instruction::Div);
                Ok(instructions)
            }
            Expression::Mod(value_0, value_1) => {
                self.check_if_initialised(value_0.clone());
                self.check_if_initialised(value_1.clone());
                let mut instructions = self.extract_value(value_0)?;
                instructions.push(Instruction::Put(B));
                instructions.extend(self.extract_value(value_1)?);
                instructions.push(Instruction::Put(C));
                instructions.push(Instruction::Mod);
                Ok(instructions)
            }
        }
    }

    /// Generates instructions to load the address of a variable into a register.
    /// Handles different types of identifiers: base, numerically indexed, and procedurally indexed.
    fn load_variable_address(&self, identifier: Identifier) -> Result<Vec<Instruction>, CompilerError> {
        match identifier {
            Identifier::Base(id) => self.access_common_variable(id),
            Identifier::NumIndexed(id, num) => self.access_array_element(id, num as usize),
            Identifier::PidIndexed(id, index_id) => self.access_dynamic_index_element(id, index_id),
        }
    }

    /// Generates instructions for accessing a base identifier's value.
    /// It checks if the identifier is a simple variable and returns instructions
    /// to put its memory address in a register, handling undeclared and incorrectly used variables.
    fn access_common_variable(&self, id: (String, usize)) -> Result<Vec<Instruction>, CompilerError> {
        let variable = self.memory.get(&id.0)
            .ok_or(CompilerError::UndeclaredVariable(id.0.clone(), id.1))?;
        match variable {
            VariableVariant::Atomic(pointer) => Ok(put_in_a(*pointer)),
            VariableVariant::Table(_, _) => Err(CompilerError::IncorrectUseOfVariable(id.0, id.1)),
        }
    }

    /// Generates instructions for accessing an element of an array by a numerical index.
    /// It calculates the memory address of the element and handles errors like undeclared variables
    /// or index out of bounds.
    fn access_array_element(&self, id: (String, usize), num: usize) -> Result<Vec<Instruction>, CompilerError> {
        let variable = self.memory.get(&id.0)
            .ok_or(CompilerError::UndeclaredVariable(id.0.clone(), id.1))?;
        match variable {
            VariableVariant::Atomic(_) => Err(CompilerError::IncorrectUseOfVariable(id.0, id.1)),
            VariableVariant::Table(pointer, size) => {
                if num >= *size as usize {
                    Err(CompilerError::IndexOutOfBounds(id.0, id.1))
                } else {
                    Ok(put_in_a(pointer + num as u64))
                }
            }
        }
    }

    /// Generates instructions for accessing an array element indexed by another variable.
    /// It calculates the element's memory address using the index variable's value,
    /// handling various errors such as undeclared variables or using an array as an index.
    fn access_dynamic_index_element(&self, id: (String, usize), index_id: (String, usize)) -> Result<Vec<Instruction>, CompilerError> {
        if !self.initialisated_variables.contains(&index_id.0) {
            let warning_id = index_id.0.split('@').next().unwrap().to_string();
            println!("Warning: Variable {} used before initialisation", warning_id);
        }

        let variable = self.memory.get(&index_id.0)
            .ok_or(CompilerError::UndeclaredVariable(index_id.0.clone(), index_id.1))?;
        let mut instructions = match variable {
            VariableVariant::Atomic(pointer) => put_in_a(*pointer),
            VariableVariant::Table(_, _) => return Err(CompilerError::ArrayUsedAsIndex(id.0, id.1)),
        };

        instructions.push(Instruction::Load(A));
        instructions.push(Instruction::Put(H));

        match self.memory.get(&id.0).unwrap() {
            VariableVariant::Atomic(_) => Err(CompilerError::IncorrectUseOfVariable(id.0, id.1)),
            VariableVariant::Table(pointer, _) => {
                instructions.extend(put_in_a(*pointer));
                instructions.push(Instruction::Add(H));
                Ok(instructions)
            }
        }
    }
    fn extract_value(&self, value: Value) -> Result<Vec<Instruction>, CompilerError> {
        match value {
            Value::Num(num) => Ok(put_in_a(num)),
            Value::Id(identifier) => {
                let mut sub_instructions = self.load_variable_address(identifier)?;
                sub_instructions.push(Instruction::Load(A));
                Ok(sub_instructions)
            }
        }
    }
    fn check_if_initialised(&self, value: Value) {
        match value {
            Value::Num(_) => {},
            Value::Id(identifier) => {
                let id = match identifier.clone() {
                    Identifier::Base(id) => id,
                    Identifier::NumIndexed(id, _) => id,
                    Identifier::PidIndexed(id, _) => id,
                };
                if !self.initialisated_variables.contains(&id.0) {
                    let id_for_warning = id.0.split('@').next().unwrap().to_string();
                    println!("Warning: Variable {} used before initialisation", id_for_warning)
                }
            },
        }
    }
    fn construct_main(&mut self) -> Result<(), CompilerError> {
        let commands = self.ast.1 .1.clone();

        let constructed_commands: Vec<Instruction> = commands
            .into_iter()
            .map(|command| self.make_instructions_list(command))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        self.pseudo_assembly.extend(constructed_commands);

        Ok(())
    }

    /// Constructs a sequence of instructions from a given command.
    /// This function handles different types of commands (e.g., Assign, If, While, Repeat, ProcCall, Read, Write)
    /// by converting them into a list of pseudo-assembly instructions.
    fn make_instructions_list(&mut self, command: Command) -> Result<Vec<Instruction>, CompilerError> {
        match command {
            Command::Assign(identifier, expression) => {
                let mut instructions: Vec<Instruction> = Vec::new();
                let id = match identifier.clone() {
                    Identifier::Base(id) => id,
                    Identifier::NumIndexed(id, _) => id,
                    Identifier::PidIndexed(id, _) => id,
                };
                self.initialisated_variables.insert(id.0.clone());
                instructions.extend(self.load_variable_address(identifier)?);
                instructions.push(Instruction::Put(G));
                instructions.extend(self.make_expressions(expression)?);
                instructions.push(Instruction::Store(G));
                Ok(instructions)
            }
            Command::If(condition, commands, else_commands) => {
                let mut instructions: Vec<Instruction> = Vec::new();
                let mut sub_instuctions: Vec<Instruction> = Vec::new();
                for command in commands {
                    sub_instuctions.extend(self.make_instructions_list(command)?);
                }
                let sub_instructions_length: u64 = sub_instuctions.iter().map(|i| i.len()).sum();
                let mut sub_else_instuctions: Vec<Instruction> = Vec::new();
                if let Some(else_commands) = else_commands {
                    for command in else_commands {
                        sub_else_instuctions.extend(self.make_instructions_list(command)?);
                    }
                }
                let sub_else_instruction_length: u64 =
                    sub_else_instuctions.iter().map(|i| i.len()).sum();
                match condition {
                    Condition::Equal(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 5));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_instuctions);
                        instructions
                            .push(Instruction::Jump(sub_else_instruction_length as i64 + 1));
                        instructions.extend(sub_else_instuctions);
                    }
                    Condition::NotEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_else_instruction_length as i64 + 5));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions
                            .push(Instruction::Jpos(sub_else_instruction_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_else_instuctions);
                        instructions.push(Instruction::Jump(sub_instructions_length as i64 + 1));
                        instructions.extend(sub_instuctions);
                    }
                    Condition::Greater(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_else_instruction_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_else_instuctions);
                        instructions.push(Instruction::Jump(sub_instructions_length as i64 + 1));
                        instructions.extend(sub_instuctions);
                    }
                    Condition::Lower(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_else_instruction_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_else_instuctions);
                        instructions.push(Instruction::Jump(sub_instructions_length as i64 + 1));
                        instructions.extend(sub_instuctions);
                    }
                    Condition::GreaterOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_instuctions);
                        instructions
                            .push(Instruction::Jump(sub_else_instruction_length as i64 + 1));
                        instructions.extend(sub_else_instuctions);
                    }
                    Condition::LowerOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));

                        instructions.extend(cond_instructions);
                        instructions.extend(sub_instuctions);
                        instructions
                            .push(Instruction::Jump(sub_else_instruction_length as i64 + 1));
                        instructions.extend(sub_else_instuctions);
                    }
                }
                Ok(instructions)
            }
            Command::While(condition, commands) => {
                let mut instructions: Vec<Instruction> = Vec::new();
                let mut sub_instuctions: VecDeque<Instruction> = VecDeque::new();
                for command in commands {
                    sub_instuctions.extend(self.make_instructions_list(command)?);
                }
                let sub_instructions_length: u64 = sub_instuctions.iter().map(|i| i.len()).sum();
                let cond_instructions = match condition {
                    Condition::Equal(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 5));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                    Condition::NotEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions.push(Instruction::Jpos(5));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions.push(Instruction::Jpos(2));
                        cond_instructions
                            .push(Instruction::Jump(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                    Condition::Greater(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions.push(Instruction::Jpos(2));
                        cond_instructions
                            .push(Instruction::Jump(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                    Condition::Lower(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions.push(Instruction::Jpos(2));
                        cond_instructions
                            .push(Instruction::Jump(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                    Condition::GreaterOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                    Condition::LowerOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions
                            .push(Instruction::Jpos(sub_instructions_length as i64 + 2));
                        cond_instructions
                    }
                };
                let cond_instructions_length: u64 = cond_instructions.iter().map(|i| i.len()).sum();
                instructions.extend(cond_instructions);
                instructions.extend(sub_instuctions);
                instructions.push(Instruction::Jump(
                    -((sub_instructions_length + cond_instructions_length) as i64),
                ));
                Ok(instructions)
            }
            Command::Repeat(commands, condition) => {
                let mut instructions: Vec<Instruction> = Vec::new();
                let mut sub_instuctions: VecDeque<Instruction> = VecDeque::new();
                for command in commands {
                    sub_instuctions.extend(self.make_instructions_list(command)?);
                }
                let sub_instructions_length: u64 = sub_instuctions.iter().map(|i| i.len()).sum();

                let cond_instructions = match condition {
                    Condition::Equal(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jpos(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions.push(Instruction::Jpos(
                            -((cond_instructions_length + sub_instructions_length + 3) as i64),
                        ));
                        cond_instructions
                    }
                    Condition::NotEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(C));
                        cond_instructions.push(Instruction::Sub(B));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jpos(5));
                        cond_instructions.push(Instruction::Get(B));
                        cond_instructions.push(Instruction::Sub(C));
                        cond_instructions.push(Instruction::Jpos(2));
                        cond_instructions.push(Instruction::Jump(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions
                    }
                    Condition::Greater(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions.push(Instruction::Jpos(2));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jump(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions
                    }
                    Condition::Lower(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        cond_instructions.push(Instruction::Jpos(2));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jump(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions
                    }
                    Condition::GreaterOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Sub(B));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jpos(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions
                    }
                    Condition::LowerOrEqual(value_0, value_1) => {
                        let mut cond_instructions: Vec<Instruction> = Vec::new();
                        cond_instructions.extend(self.extract_value(value_1)?);
                        cond_instructions.push(Instruction::Put(B));
                        cond_instructions.extend(self.extract_value(value_0)?);
                        cond_instructions.push(Instruction::Sub(B));
                        let cond_instructions_length: u64 =
                            cond_instructions.iter().map(|i| i.len()).sum();
                        cond_instructions.push(Instruction::Jpos(
                            -((cond_instructions_length + sub_instructions_length) as i64),
                        ));
                        cond_instructions
                    }
                };
                instructions.extend(sub_instuctions);
                instructions.extend(cond_instructions);
                Ok(instructions)
            }
            Command::ProcCall((procedure_id, arguments)) => {
                let mut instructions: Vec<Instruction> = Vec::new();
                let ids: Vec<String> = arguments.iter().map(|arg| arg.0.clone()).collect();
                for id in ids {
                    if  (&id).contains(&format!("@{}", procedure_id.0)) {
                        return Err(CompilerError::RecursiveProcedureCall(procedure_id.0, procedure_id.1));
                    }
                }

                let builder = self.procedures.clone().get(&procedure_id.0).ok_or(CompilerError::UndeclaredProcedure(procedure_id.0.clone(), procedure_id.1))?.clone();
                if builder.declared_arguments.len() != arguments.len() {
                    return Err(CompilerError::WrongNumberOfArguments(procedure_id.0.clone(), procedure_id.1));
                }
                if let Some(declarations) = &builder.declarations {
                    for declaration in declarations {
                        match declaration {
                            DeclarationVariant::Base(id) => {
                                self.memory.insert(format!("{}@{}", id.0, procedure_id.0), VariableVariant::Atomic(self.memory_pointer));
                                self.memory_pointer += 1;
                            },
                            DeclarationVariant::NumIndexed(id, length) => {
                                self.memory.insert(format!("{}@{}", id.0, procedure_id.0), VariableVariant::Table(self.memory_pointer, *length));
                                self.memory_pointer += length;
                            },
                        }
                    }
                }

                for (argument, declared_argument) in arguments.iter().zip(&builder.declared_arguments) {
                    if let Some(declarations) = &builder.declarations{
                        for declaration in declarations {
                            let id = match declaration {
                                DeclarationVariant::Base(id) => id,
                                DeclarationVariant::NumIndexed(id, _) => id,
                            };
                            let arg_id = match declared_argument {
                                ArgumentsDeclarationVariant::Base(id) => id,
                                ArgumentsDeclarationVariant::Table(id) => id,
                            };
                            if id.0 == arg_id.0 {
                                return Err(CompilerError::DuplicateVariableDeclaration(id.0.clone(), id.1));
                            }
                        }
                    }
                    let pointee = self.memory.get(argument.0.as_str()).unwrap();
                    match declared_argument {
                        ArgumentsDeclarationVariant::Base(id) => {
                            match pointee {
                                VariableVariant::Atomic(pointer) => {
                                    self.initialisated_variables.insert(argument.0.clone());
                                    self.memory.insert(format!("{}@{}", id.0, procedure_id.0), VariableVariant::Atomic(*pointer));
                                    self.initialisated_variables.insert(format!("{}@{}", id.0, procedure_id.0));
                                },
                                VariableVariant::Table(_, _) => return Err(CompilerError::WrongArgumentType(id.0.clone(), id.1)),
                            }
                        },
                        ArgumentsDeclarationVariant::Table(id) => {
                            match pointee {
                                VariableVariant::Atomic(_) => return Err(CompilerError::WrongArgumentType(id.0.clone(), id.1)),
                                VariableVariant::Table(start, size) => {
                                    self.initialisated_variables.insert(argument.0.clone());
                                    self.memory.insert(format!("{}@{}", id.0, procedure_id.0), VariableVariant::Table(*start, *size));
                                    self.initialisated_variables.insert(format!("{}@{}", id.0, procedure_id.0));
                                },
                            }
                        },
                    }
                }
                for command in &builder.commands {
                    instructions.extend(self.make_instructions_list(command.clone())?);
                }
                Ok(instructions)
            }
            Command::Read(identifier) => {
                let id = match identifier.clone() {
                    Identifier::Base(id) => id,
                    Identifier::NumIndexed(id, _) => id,
                    Identifier::PidIndexed(id, _) => id,
                };
                self.initialisated_variables.insert(id.0.clone());
                let mut instructions: Vec<Instruction> = Vec::new();
                instructions.extend(self.load_variable_address(identifier)?);
                instructions.push(Instruction::Put(G));
                instructions.push(Instruction::Read);
                instructions.push(Instruction::Store(G));
                Ok(instructions)
            }
            Command::Write(value) => {
                let mut instructions: Vec<Instruction> = self.extract_value(value)?;
                instructions.push(Instruction::Write);
                Ok(instructions)
            }
        }
    }

}


