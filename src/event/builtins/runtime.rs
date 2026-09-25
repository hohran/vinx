use crate::action::ActionHandle;
use crate::video::Drawable;

use super::*;
use context::Context;

/// Function callable only at runtime.
/// This allows the function to access the current frame or change the flow of program with Handles.
pub type BuiltinRuntime = fn(&mut context::Runtime, &mut Vec<Variable>) -> Option<VariableValue>;

pub mod image {
    use crate::variable::Color;

    use super::*;
    use ::image;

    pub fn draw_at(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("draw image at", params, 2);
        // if context.is_empty() { return None; }
        let par1 = context.get_value(&params[0]);
        let par2 = context.get_value(&params[1]);
        let img = par1.into_image().clone(); // TODO: uff we should be able not to clone this
        let pos = par2.into_pos();
        let frame = context.get_current_frame_mut();
        image::imageops::overlay(frame, &img, pos.x.into(), pos.y.into());
        None
    }
}

pub mod rectangle {
    use crate::variable::Rectangle;

    use super::*;

    pub fn draw(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("draw struct rectangle", params, 2);
        if context.is_empty() { return None; }
        let par1 = context.get_value(&params[0]);
        let par2 = context.get_value(&params[1]);
        let c = par1.into_color();
        let r = par2.into_rectangle();
        let top_left = r.top_left;
        let bot_right = r.bot_right;
        let frame = context.get_current_frame_mut();
        frame.draw_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
        None
    }

    pub fn draw_outline(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("draw rectangle outline", params, 2);
        if context.is_empty() { return None; }
        let par1 = context.get_value(&params[0]);
        let par2 = context.get_value(&params[1]);
        let c = par1.into_color();
        let r = par2.into_rectangle();
        let top_left = r.top_left;
        let bot_right = r.bot_right;
        let frame = context.get_current_frame_mut();
        frame.draw_rect_outline((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
        None
    }
}

pub mod column {
    use crate::{variable::Column, video::Extendable};

    use super::*;

    pub fn take(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("take column", params, 1);
        let at = context.get_value(&params[0]).into_int();
        let frame = context.get_current_frame();
        if at < 0 {
            panic!("error: attempted to take negative column")
        }
        let col = Column::take(frame, at as u32);
        Some(VariableValue::Column(col))
    }
}

pub mod row {
    use crate::variable::Row;

    use super::*;

    pub fn take(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("take column", params, 1);
        let at = context.get_value(&params[0]).into_int();
        let frame = context.get_current_frame();
        if at < 0 {
            panic!("error: attempted to take negative column")
        }
        let row = Row::take(frame, at as u32);
        Some(VariableValue::Row(row))
    }
}

pub fn activate(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    let op_name = "activate";
    expect_param_count(op_name, params, 1);
    let par1 = context.get_value(&params[0]);
    let label = par1.into_string();
    context.add_handle(ActionHandle::Enable(label.to_string()));
    None
}

pub fn stop(context: &mut context::Runtime, _params: &mut Vec<Variable>) -> Option<VariableValue> {
    context.add_handle(ActionHandle::Stop);
    None
}

pub fn deactivate(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    let op_name = "deactivate";
    expect_param_count(op_name, params, 1);
    let par1 = context.get_value(&params[0]);
    let label = par1.into_string();
    context.add_handle(ActionHandle::Disable(label.to_string()));
    None
}

pub fn toggle_activeness(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    let op_name = "toggle";
    expect_param_count(op_name, params, 1);
    let par1 = context.get_value(&params[0]);
    let label = par1.into_string();
    context.add_handle(ActionHandle::Toggle(label.to_string()));
    None
}

pub fn get_frame(context: &mut context::Runtime, _params: &mut Vec<Variable>) -> Option<VariableValue> {
    if context.is_empty() { 
        panic!("error: cannot return frame when context is empty");
    }
    let frame = context.get_current_frame();
    Some(VariableValue::Image(frame.clone()))
}

pub fn draw_rect(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = context.get_value(&params[0]);
    let par2 = context.get_value(&params[1]);
    let par3 = context.get_value(&params[2]);
    let c = par1.into_color();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
    None
}

pub fn draw_effect_rect(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("draw rectangle (effect)", params, 3);
    if context.is_empty() { return None; }
    let par1 = context.get_value(&params[0]);
    let par2 = context.get_value(&params[1]);
    let par3 = context.get_value(&params[2]);
    let e = par1.into_effect();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_effect_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), e);
    None
}

pub fn draw_rect_outline(context: &mut context::Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = context.get_value(&params[0]);
    let par2 = context.get_value(&params[1]);
    let par3 = context.get_value(&params[2]);
    let c = par1.into_color();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_rect_outline((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
    None
}
