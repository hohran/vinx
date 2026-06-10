use crate::translator::Sequence;
use super::Location;

pub enum CompilationError {
    TemporaryError(String),
    UnknownSequence(Sequence, Location),
    RedeclaredVariable(String, Location),
    ForbiddenVariableName(String, Location),
    UnknownVariableName(String, Location),
    FileNotFound(String, Option<Location>),
    RecursiveFileDependency(String, String, Location),
    MultipleMainIterators(Location),
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
            Self::UnknownSequence(seq, loc) => {
                print_err!("unknown sequence `{seq}`");
                eprintln!("{}", loc.get_source());
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
            Self::VagueDefinition(sign_loc, first_seq_loc, first_met_loc) => {
                print_err!("definition is neither operation, nor structure");
                eprint!("{}", sign_loc.get_source());
                print_note!("sequences imply it should be an operation:");
                eprint!("{}", first_seq_loc.get_concise_source());
                print_note!("nested definitions imply it should be a structure:");
                eprintln!("{}", first_met_loc.get_concise_source());
            }
        }
    }
}
