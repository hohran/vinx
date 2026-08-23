use crate::translator::Signature;

use super::Location;

pub enum Warning {
    RedundantFileLoad(String, Location),
    OperationWithoutInterpretation(Signature),
    StructureWithoutInterpretation(Signature),
    ExistingSignature(Signature),
}

impl Warning {
    pub fn print(&self) {
        eprint!("warning: "); // TODO: colorize
        match self {
            Self::RedundantFileLoad(fp, loc) => {
                eprintln!("file `{fp}` already loaded: skipping");
                eprintln!("{}", loc.get_source());
            }
            Self::OperationWithoutInterpretation(sig) => {
                eprintln!("operation `{sig}` does not have any interpretation");
                eprintln!("{}", sig.get_location().get_source());
            }
            Self::StructureWithoutInterpretation(sig) => {
                eprintln!("structure `{sig}` does not have any interpretation");
                eprintln!("{}", sig.get_location().get_source());
            }
            Self::ExistingSignature(sig) => {
                eprintln!("signature `{}` already exists", sig.sequence);
                eprintln!("{}", sig.get_location().get_source());
            }
        }
    }
}

