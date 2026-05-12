use super::Action;

/// Events return these handles, which modify the actions.
#[derive(Debug)]
pub enum ActionHandle {
    Enable(String),
    Disable(String),
    Toggle(String),
}

impl ActionHandle {
    pub fn get_action_name(&self) -> &str {
        match self {
            ActionHandle::Enable(n) => n,
            ActionHandle::Disable(n) => n,
            ActionHandle::Toggle(n) => n,
        }
    }

    pub fn trigger(&self, actions: &mut Vec<Action>) {
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
            }
        }
    }
}

pub fn process_action_handles(handles: &mut Vec<ActionHandle>, actions: &mut Vec<Action>) {
    for h in handles.iter() {
        h.trigger(actions);
    }
    handles.clear();
}
