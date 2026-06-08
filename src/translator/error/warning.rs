use super::Location;

pub enum Warning {
    RedundantFileLoad(String, Location),
}

impl Warning {
    pub fn print(&self) {
        eprint!("warning: "); // TODO: colorize
        match self {
            Self::RedundantFileLoad(fp, loc) => {
                eprintln!("file `{fp}` already loaded: skipping");
                eprintln!("{}", loc.get_source());
            }
        }
    }
}

