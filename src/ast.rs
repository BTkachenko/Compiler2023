pub type Num = u64;

pub type SourceIdent = (String, usize);

#[derive(Debug, Clone)]
pub enum Identifier {
    Base(SourceIdent),
    NumIndexed(SourceIdent, Num),
    PidIndexed(SourceIdent, SourceIdent),
}

#[derive(Debug, Clone)]
pub enum Value {
    Num(Num),
    Id(Identifier),
}
#[derive(Debug, Clone)]
pub enum Condition {
    Equal(Value, Value),
    NotEqual(Value, Value),
    Greater(Value, Value),
    Lower(Value, Value),
    GreaterOrEqual(Value, Value),
    LowerOrEqual(Value, Value),
}

#[derive(Debug, Clone)]
pub enum Command {
    Assign(Identifier, Expression),
    If(Condition, Commands, Option<Commands>),
    While(Condition, Commands),
    Repeat(Commands, Condition),
    ProcCall(ProcedureCall),
    Read(Identifier),
    Write(Value),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    Mod(Value, Value),
}

#[derive(Debug, Clone)]
pub enum ArgumentsDeclarationVariant {
    Base(SourceIdent),
    Table(SourceIdent),
}

#[derive(Debug, Clone)]
pub enum DeclarationVariant {
    Base(SourceIdent),
    NumIndexed(SourceIdent, Num),
}

pub type Arguments = Vec<SourceIdent>;



pub type ArgumentsDeclaration = Vec<ArgumentsDeclarationVariant>;



pub type Declarations = Vec<DeclarationVariant>;

pub type ProcedureCall = (SourceIdent, Arguments);

pub type ProcedureHead = (SourceIdent, ArgumentsDeclaration);


pub type Commands = Vec<Command>;

pub type Main = (Option<Declarations>, Commands);

pub type Procedure = (ProcedureHead, Option<Declarations>, Commands);

pub type Procedures = Vec<Procedure>;

pub type Program = (Option<Procedures>, Main);
