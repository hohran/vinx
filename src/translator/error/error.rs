use crate::{translator::{Sequence, Signature, parser::Expression}, variable::VariableType};
use super::Location;

pub enum CompilationError {
    TemporaryError(String),
    UnexpectedType(String, VariableType, VariableType, Location), // 1) variable, 2) got, 
                                                                  // 3) expected, 4) location
    UnknownSequence(Sequence),
    RedeclaredVariable(String, Location),
    ForbiddenVariableName(String, Location),
    UnknownVariableName(String, Location),
    FileNotFound(String, Option<Location>),
    MemberUsedBeforeDefinition(String, Location),
    DuplicateMemberName(String, Location, Location),
    AssignmentOfUndefinedVariable(String, Location),
    RecursiveFileDependency(String, String, Location),
    MultipleMainIterators(Location),
    UnboundMethod(Signature),
    HeterogenousVector(Expression, Expression),
    VagueDefinition(Location, Location, Location), // the definition is neither structure nor operation
                                                   // the params are: 1) signature, 2) first sequence, 3) first method
}

macro_rules! print_err {
    ($($arg:tt)*) => {
        eprintln!("error: {}", format!($($arg)*)); // TODO: colorize
    };
}

macro_rules! print_note {
    ($($arg:tt)*) => {
        eprintln!("note: {}", format!($($arg)*)); // TODO: colorize
    };
}

impl CompilationError {
    pub fn print(&self) {
        match self {
            Self::TemporaryError(s) => {
                print_err!("{s}");
            }
            Self::HeterogenousVector(e1, e2) => {
                print_err!("expressions `{e1}` and `{e2}` have distinct types");
                // TODO: print location
                print_note!("vectors can only contain elements of the same type");
            }
            Self::AssignmentOfUndefinedVariable(name, loc) => {
                print_err!("assignment into an undefined variable `{name}`");
                eprint!("{}", loc.get_source());
                print_note!("you probably wished to define it (with the `:=` symbol)");
            }
            Self::UnboundMethod(sig) => {
                print_err!("signature `{sig}` is not bound to the structure");
                eprint!("{}", sig.get_location().get_source());
                print_note!("use the builtin reference name `$self` in the method signature"); // TODO: use $self by variable (in case we change it)
                print_note!("for example: `{sig} for $self`");
            }
            Self::UnexpectedType(name, got, expected, loc) => {
                print_err!("expected `{name}` to have type `{expected}`, got `{got}`");
                eprintln!("{}", loc.get_source());
            }
            Self::UnknownSequence(seq) => {
                print_err!("unknown sequence `{seq}`");
                eprintln!("{}", seq.get_location().get_source());
            }
            Self::RedeclaredVariable(s, loc) => {
                print_err!("redeclared variable name `{s}`");
                eprintln!("{}", loc.get_source());
            }
            Self::UnknownVariableName(n, loc) => {
                print_err!("unknown variable name `{n}`");
                eprintln!("{}", loc.get_source());
            }
            Self::ForbiddenVariableName(s, loc) => {
                print_err!("forbidden variable name `{s}`");
                eprintln!("{}", loc.get_source());
            }
            Self::FileNotFound(fp, loc) => {
                print_err!("file `{fp}` not found");
                if let Some(loc) = loc {
                    eprintln!("{}", loc.get_source());
                }
            }
            Self::RecursiveFileDependency(fp1, fp2, loc) => {
                print_err!("files `{fp1}` and `{fp2}` are recursively dependent on each other");
                eprintln!("{}", loc.get_source());
            }
            Self::MultipleMainIterators(loc) => {
                print_err!("multiple iterators set as main");
                eprintln!("{}", loc.get_source());
            }
            Self::DuplicateMemberName(name, loc1, first_loc) => {
                print_err!("duplicate definition of local variable `{name}`");
                eprint!("{}", loc1.get_source());
                print_note!("first defined here:");
                eprint!("{}", first_loc.get_concise_source());
            }
            Self::VagueDefinition(sign_loc, first_seq_loc, first_met_loc) => {
                print_err!("definition is neither operation, nor structure");
                eprint!("{}", sign_loc.get_source());
                print_note!("sequences imply it should be an operation:");
                eprint!("{}", first_seq_loc.get_concise_source());
                print_note!("nested definitions imply it should be a structure:");
                eprintln!("{}", first_met_loc.get_concise_source());
            }
            Self::MemberUsedBeforeDefinition(name, loc) => {
                print_err!("variable {name} is used before its definition");
                eprint!("{}", loc.get_source());
                print_note!("if you wish to access a global variable of the same name, rename one of them");
            }
        }
    }
}
