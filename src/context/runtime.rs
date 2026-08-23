use crate::{action::ActionHandle, variable::{Scope, Stack, Variable, VariableValue}, video::{Frame, VideoReader}};
use super::Compiletime;

pub struct Runtime<'a> {
    video_reader: VideoReader<'a>,
    current_frame: Option<Frame>,
    stack: Stack,
    handles: Vec<ActionHandle>,
}

impl<'a> Runtime<'a> {
    pub fn is_empty(&self) -> bool {
        self.current_frame.is_none()
    }

    pub fn new(video_reader: VideoReader<'a>, stack: Stack) -> Self {
        Self { video_reader, current_frame: None, stack, handles: vec![] }
    }

    pub fn from(video_reader: VideoReader<'a>, compilation_context: Compiletime) -> Self {
        Self { video_reader, current_frame: None, stack: compilation_context.into_stack(), handles: vec![] }
    }

    pub fn add_handle(&mut self, handle: ActionHandle) {
        self.handles.push(handle);
    }

    pub fn get_frame_index(&self) -> usize {
        self.video_reader.get_frame_index()
    }

    pub fn load_next_frame(&mut self) -> bool {
        self.current_frame = self.video_reader.get_next_frame();
        self.current_frame.is_some()
    }

    pub fn pop_current_frame(&mut self) -> Frame {
        self.current_frame.take().expect("error: no current frame loaded")
    }

    pub fn get_current_frame(&self) -> &Frame {
        self.current_frame.as_ref().expect("error: no current frame loaded")
    }

    pub fn get_current_frame_mut(&mut self) -> &mut Frame {
        self.current_frame.as_mut().expect("error: no current frame loaded")
    }

    pub fn get_width(&self) -> usize {
        self.video_reader.width() as usize
    }

    pub fn get_height(&self) -> usize {
        self.video_reader.height() as usize
    }

}

impl super::Context for Runtime<'_> {
    fn get_value<'c>(&'c self, v: &'c Variable) -> &'c VariableValue {
        v.get_value(&self.stack)
    }

    fn get_value_mut<'c>(&'c mut self, v: &'c mut Variable) -> &'c mut VariableValue {
        v.get_value_mut(&mut self.stack)
    }

    fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        self.stack.get_variable(name)
    }

    fn set_value(&mut self, v: &mut Variable, new_value: VariableValue) {
        v.set_value(&mut self.stack, new_value);
    }

    fn update_variable(&mut self, name: &str, new_value: VariableValue) {
        self.stack.update_variable(name, new_value);
    }

    fn push_scope(&mut self) {
        self.stack.push();
    }

    fn push_scope_with(&mut self, scope: Scope) {
        self.stack.push_scope(scope);
    }

    fn pop_scope(&mut self) {
        self.stack.pop();
    }

    fn add_variable(&mut self, name: String, value: VariableValue) -> bool {
        self.stack.add_variable(name, value)
    }

    fn get_stack(&self) -> &Stack {
        &self.stack
    }

    fn get_stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}
