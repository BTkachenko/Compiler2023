use crate::ast::{
ArgumentsDeclarationVariant, Command, Commands, Condition, Declarations, Expression,
Identifier, Procedure, Value,
};
use crate::emitter::Registers;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Jumps are relative in our pseudo-Instructions
pub enum Instruction {
    Read,
    Write,
    Load(Registers),
    Store(Registers),
    Add(Registers),
    Sub(Registers),
    Get(Registers),
    Put(Registers),
    Rst(Registers),
    Inc(Registers),
    Dec(Registers),
    Shl(Registers),
    Shr(Registers),
    Strk,
    Jumpr(Registers),
    Jump(i64),
    Jpos(i64),
    Jzero(i64),
    Halt,
    Mul,
    Div,
    Mod,
}



#[derive(Debug, Clone)]
pub struct ProcedureBuilder {
    name: String,
    pub(crate) declared_arguments: Vec<ArgumentsDeclarationVariant>,
    pub(crate) declarations: Option<Declarations>,
    pub(crate) commands: Commands,
}

impl ProcedureBuilder {
    pub fn new(procedure: Procedure) -> Self {
        let mut pb = Self {
            name: procedure.0 .0 .0,
            declared_arguments: procedure.0 .1,
            declarations: procedure.1,
            commands: procedure.2,
        };
        pb.rename_commands();
        pb
    }

    fn rename_commands(&mut self) {
        self.commands = self
            .commands
            .iter()
            .cloned()
            .map(|com| self.rename_command(com))
            .collect();
    }
    fn rename_command(&self, command: Command) -> Command {
        match command {
            Command::Assign(id, expression) => {
                let new_id = self.rename_indentifier(id);
                let new_expression = match expression {
                    Expression::Value(value) => {
                        let new_value = self.rename_value(value);
                        Expression::Value(new_value)
                    }
                    Expression::Add(value0, value1) => {
                        let new_value0 = self.rename_value(value0);
                        let new_value1 = self.rename_value(value1);
                        Expression::Add(new_value0, new_value1)
                    }
                    Expression::Sub(value0, value1) => {
                        let new_value0 = self.rename_value(value0);
                        let new_value1 = self.rename_value(value1);
                        Expression::Sub(new_value0, new_value1)
                    }
                    Expression::Mul(value0, value1) => {
                        let new_value0 = self.rename_value(value0);
                        let new_value1 = self.rename_value(value1);
                        Expression::Mul(new_value0, new_value1)
                    }
                    Expression::Div(value0, value1) => {
                        let new_value0 = self.rename_value(value0);
                        let new_value1 = self.rename_value(value1);
                        Expression::Div(new_value0, new_value1)
                    }
                    Expression::Mod(value0, value1) => {
                        let new_value0 = self.rename_value(value0);
                        let new_value1 = self.rename_value(value1);
                        Expression::Mod(new_value0, new_value1)
                    }
                };
                Command::Assign(new_id, new_expression)
            }
            Command::If(condition, commands, else_commands) => {
                let new_condition = self.rename_condition(condition);
                let new_commands: Vec<Command> = commands
                    .iter()
                    .cloned()
                    .map(|com| self.rename_command(com))
                    .collect();
                let new_else_condition: Option<Vec<Command>> = else_commands.map(|else_commands| else_commands
                            .iter()
                            .cloned()
                            .map(|com| self.rename_command(com))
                            .collect());
                Command::If(new_condition, new_commands, new_else_condition)
            }
            Command::While(condition, commands) => {
                let new_condition = self.rename_condition(condition);
                let new_commands: Vec<Command> = commands
                    .iter()
                    .cloned()
                    .map(|com| self.rename_command(com))
                    .collect();
                Command::While(new_condition, new_commands)
            }
            Command::Repeat(commands, condition) => {
                let new_condition = self.rename_condition(condition);
                let new_commands: Vec<Command> = commands
                    .iter()
                    .cloned()
                    .map(|com| self.rename_command(com))
                    .collect();
                Command::Repeat(new_commands, new_condition)
            }
            Command::ProcCall((name, arguments)) => {
                let new_arguments: Vec<(String, usize)> = arguments.iter().map(|arg| (format!("{}@{}", arg.0, self.name), arg.1)).collect();
                Command::ProcCall((name, new_arguments))
            },
            Command::Read(identifier) => {
                let new_identifier = self.rename_indentifier(identifier);
                Command::Read(new_identifier)
            },
            Command::Write(value) => {
                let new_value = self.rename_value(value);
                Command::Write(new_value)
            },
        }
    }
    fn rename_condition(&self, condition: Condition) -> Condition {
        match condition {
            Condition::Equal(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::Equal(new_value0, new_value1)
            }
            Condition::NotEqual(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::NotEqual(new_value0, new_value1)
            }
            Condition::Greater(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::Greater(new_value0, new_value1)
            }
            Condition::Lower(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::Lower(new_value0, new_value1)
            }
            Condition::GreaterOrEqual(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::GreaterOrEqual(new_value0, new_value1)
            }
            Condition::LowerOrEqual(value0, value1) => {
                let new_value0 = self.rename_value(value0);
                let new_value1 = self.rename_value(value1);
                Condition::LowerOrEqual(new_value0, new_value1)
            }
        }
    }
    fn rename_value(&self, value: Value) -> Value {
        match value {
            Value::Num(_) => value.clone(),
            Value::Id(id) => Value::Id(self.rename_indentifier(id)),
        }
    }
    fn rename_indentifier(&self, identifier: Identifier) -> Identifier {
        match identifier {
            Identifier::Base(id) => Identifier::Base((format!("{}@{}", id.0, self.name), id.1)),
            Identifier::NumIndexed(id, num) => {
                Identifier::NumIndexed((format!("{}@{}", id.0, self.name), id.1), num)
            }
            Identifier::PidIndexed(id, index_id) => Identifier::PidIndexed(
                (format!("{}@{}", id.0, self.name), id.1),
                (format!("{}@{}", index_id.0, self.name), id.1),
            ),
        }
    }
}

impl Instruction {
    pub(crate) fn len(&self) -> u64 {
        match self {
            Instruction::Mul => 20,
            Instruction::Div => 25,
            Instruction::Mod => 26,
            _ => 1,
        }
    }
}