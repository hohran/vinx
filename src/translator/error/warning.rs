use crate::translator::Signature;

use super::Location;

pub enum Warning {
    RedundantFileLoad(String, Location),
    OperationWithoutInterpretation(Signature, Location),
}

impl Warning {
    pub fn print(&self) {
        eprint!("warning: "); // TODO: colorize
        match self {
            Self::RedundantFileLoad(fp, loc) => {
                eprintln!("file `{fp}` already loaded: skipping");
                eprintln!("{}", loc.get_source());
            }
            Self::OperationWithoutInterpretation(sig, loc) => {
                eprintln!("operation `{sig}` does not have any interpretation");
                eprintln!("{}", loc.get_source());
            }
        }
    }
}

