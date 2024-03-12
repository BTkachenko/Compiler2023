#[derive(Debug, Clone)]
pub enum CompilerError {
    UndeclaredVariable(String, usize),
    UndeclaredProcedure(String, usize),
    IncorrectUseOfVariable(String, usize),
    IndexOutOfBounds(String, usize),
    ArrayUsedAsIndex(String, usize),
    WrongArgumentType(String, usize),
    DuplicateVariableDeclaration(String, usize),
    DuplicateProcedureDeclaration(String, usize),
    RecursiveProcedureCall(String, usize),
    WrongNumberOfArguments(String, usize),
}

impl CompilerError {
    pub fn get_byte(&self) -> usize {
        match self {
            CompilerError::UndeclaredVariable(_, line)
            | CompilerError::UndeclaredProcedure(_, line)
            | CompilerError::IncorrectUseOfVariable(_, line)
            | CompilerError::IndexOutOfBounds(_, line)
            | CompilerError::ArrayUsedAsIndex(_, line)
            | CompilerError::WrongArgumentType(_, line)
            | CompilerError::DuplicateVariableDeclaration(_, line)
            | CompilerError::DuplicateProcedureDeclaration(_, line)
            | CompilerError::RecursiveProcedureCall(_, line)
            | CompilerError::WrongNumberOfArguments(_, line) => *line,
        }
    }
}
