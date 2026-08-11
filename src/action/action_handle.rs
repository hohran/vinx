use super::Action;

/// Events return these handles, which modify the actions.
#[derive(Debug)]
pub enum ActionHandle {
    Enable(String),
    Disable(String),
    Toggle(String),
    Stop,
}

impl ActionHandle {
    pub fn get_action_name(&self) -> &str {
        match self {
            ActionHandle::Enable(n) => n,
            ActionHandle::Disable(n) => n,
            ActionHandle::Toggle(n) => n,
            _ => panic!("error: we should change the name of action handle because it is now not only for actions"), // FIXME TODO TODO
        }
    }

    /// returns if the whole program should stop
    pub fn trigger(&self, actions: &mut Vec<Action>) -> bool {
        if let ActionHandle::Stop = self {
            return true;
        }
        for a in actions.iter_mut() {
            let Some(name) = a.get_name() else { continue };
            if name != self.get_action_name() { continue }
            match self {
                ActionHandle::Enable(_) => a.enable(),
                ActionHandle::Disable(_) => a.disable(),
                ActionHandle::Toggle(_) => {
                    if a.is_enabled() {
                        a.disable();
                    } else {
                        a.enable();
                    }
                }
                ActionHandle::Stop => panic!("UNREACHABLE")
            }
        }
        false
    }
}

/// returns if the whole program should stop
pub fn process_action_handles(handles: &mut Vec<ActionHandle>, actions: &mut Vec<Action>) -> bool {
    for h in handles.iter() {
        if h.trigger(actions) {
            return true;
        }
    }
    handles.clear();
    false
}
