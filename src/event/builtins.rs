use crate::action::ActionHandle;
use crate::context::Context;
use crate::variable::Variable;
use crate::variable::{Stack, Direction, VariableValue};
use crate::video::Drawable;

pub type Builtin = fn(&mut Context, &mut Stack, &mut Vec<Variable>, &mut Vec<ActionHandle>) -> Option<VariableValue>;

pub struct Runtime<'a> {
    pub context: Context<'a>,
    pub stack: Stack,
    pub action_handles: Vec<ActionHandle>
}

pub fn expect_param_count(operation_name: &str, params: &Vec<Variable>, expected: usize) {
    assert_eq!(params.len(), expected, "error: function {operation_name} expected {expected} parameters, got {}", params.len());
}

pub fn print(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("print", params, 1);
    let par1 = params[0].get_value(stack);
    let s = par1.into_string();
    println!("{s}");
    None
}

pub fn activate(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    let op_name = "activate";
    expect_param_count(op_name, params, 1);
    let par1 = params[0].get_value(stack);
    let label = par1.into_string();
    action_handles.push(ActionHandle::Enable(label.to_string()));
    None
}

pub fn stop(_context: &mut Context, _stack: &mut Stack, _params: &mut Vec<Variable>, action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    action_handles.push(ActionHandle::Stop);
    None
}

pub fn deactivate(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    let op_name = "deactivate";
    expect_param_count(op_name, params, 1);
    let par1 = params[0].get_value(stack);
    let label = par1.into_string();
    action_handles.push(ActionHandle::Disable(label.to_string()));
    None
}

pub fn toggle_activeness(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    let op_name = "toggle";
    expect_param_count(op_name, params, 1);
    let par1 = params[0].get_value(stack);
    let label = par1.into_string();
    action_handles.push(ActionHandle::Toggle(label.to_string()));
    None
}

pub fn add_to(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("add", params, 2);
    let v1 = &params[0].get_value(stack);
    let v2 = &params[1].get_value(stack);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_add(i1);
    params[1].set_value(stack, VariableValue::Int(i2));
    None
}

pub fn sub(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("sub", params, 2);
    let v1 = &params[0].get_value(stack);
    let v2 = &params[1].get_value(stack);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_sub(i1);
    params[1].set_value(stack, VariableValue::Int(i2));
    None
}

pub fn set(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("set", params, 2);
    let v2 = &params[1];
    let new_val = v2.get_value(stack).clone();
    let v1 = &mut params[0];
    v1.set_value(stack, new_val);
    None
}

pub fn top_into(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("top into", params, 2);
    let par1 = &params[0].get_value(stack);
    let v = par1.into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    let top = v[0].get_value(stack).clone();
    params[1].set_value(stack, top);
    None
}

pub fn top(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("top", params, 1);
    let v = &params[0].get_value(stack).into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    Some(v[0].get_value(stack).clone())
}

pub fn rotate_vec(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("rotate", params, 3);
    let par1 = params[0].get_value(stack);
    let par2 = params[1].get_value(stack);
    let par3 = params[2].get_value(stack);
    let mut v = par1.into_vec().clone();
    if v.is_empty() { return None; }
    let d = par2.into_direction();
    let step = par3.into_int();
    match d {
        Direction::Left => {
            for _ in 0..step {
                let e = v.remove(0);
                v.push(e);
            }
        }
        Direction::Right => {
            for _ in 0..step {
                if let Some(e) = v.pop() {
                    v.insert(0, e);
                }
            }
        }
        _ => {
            panic!("error: rotate vec: vector can only be rotated to left or right");
        }
    }
    params[0].set_value(stack, VariableValue::Vec(v));
    None
}

pub fn get_frame(context: &mut Context, _stack: &mut Stack, _params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    if context.is_empty() { 
        panic!("error: cannot return frame when context is empty");
    }
    // return None; } // TODO: should we really return None in here?
    let frame = context.get_current_frame();
    Some(VariableValue::Image(frame.clone()))
}

pub fn draw_rect(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let c = par1.into_color();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
    None
}

pub fn _draw_rect(r: &mut Runtime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if r.context.is_empty() { return None; }
    let par1 = &params[0].get_value(&r.stack);
    let par2 = &params[1].get_value(&r.stack);
    let par3 = &params[2].get_value(&r.stack);
    let c = par1.into_color();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = r.context.get_current_frame_mut();
    frame.draw_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
    None
}

pub fn draw_effect_rect(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("draw rectangle (effect)", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let e = par1.into_effect();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_effect_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), e);
    None
}

pub fn draw_rect_outline(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let c = par1.into_color();
    let top_left = par2.into_pos();
    let bot_right = par3.into_pos();
    let frame = context.get_current_frame_mut();
    frame.draw_rect_outline((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
    None
}

pub fn move_pos_phase(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("move", params, 3);
    if context.is_empty() { return None; } // TODO: what to do with wrapping in preprocessing?
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let mut pos = par1.into_pos();
    let d = par2.into_direction();
    let step = par3.into_int();
    let width = context.get_width() as i32;
    let height = context.get_height() as i32;
    match d {
        Direction::Left => {
            pos.x = (pos.x-step).rem_euclid(width); 
        }
        Direction::Right => { 
            pos.x = (pos.x+step).rem_euclid(width); 
        }
        Direction::Down => { 
            pos.y = (pos.y+step).rem_euclid(height); 
        }
        Direction::Up => {
            pos.y = (pos.y-step).rem_euclid(height); 
        }
    }
    params[0].set_value(stack, VariableValue::Pos(pos));
    None
}

pub fn get_value(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    Some(params[0].get_value(stack).clone())
}

pub fn move_pos(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("restricted move", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let mut pos = par1.into_pos();
    let d = par2.into_direction();
    let step = par3.into_int();
    let width = context.get_width() as i32;
    let height = context.get_height() as i32;
    match d {
        Direction::Left =>
            pos.x = (pos.x.saturating_sub(step)).max(width),
        Direction::Right =>
            pos.x = (pos.x.saturating_add(step)).min(width),
        Direction::Down =>
            pos.y = (pos.y.saturating_add(step)).min(height),
        Direction::Up =>
            pos.y = (pos.y.saturating_sub(step)).max(height),
    }
    params[0].set_value(stack, VariableValue::Pos(pos));
    None
}

pub fn move_by(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
    expect_param_count("move by", params, 2);
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let mut pos = par1.into_pos();
    let diff = par2.into_pos();
    pos.x = pos.x.saturating_add(diff.x);
    pos.y = pos.y.saturating_add(diff.y);
    params[0].set_value(stack, VariableValue::Pos(pos));
    None
}

pub mod image {
    use crate::variable::Color;

    use super::*;
    use ::image;

    pub fn draw_at(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("draw image at", params, 2);
        if context.is_empty() { return None; }
        let par1 = params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let img = par1.into_image();
        let pos = par2.into_pos();
        let frame = context.get_current_frame_mut();
        image::imageops::overlay(frame, img, pos.x.into(), pos.y.into());
        None
    }

    pub fn draw_into(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("draw into image", params, 3);
        let par1 = params[0].get_value(stack);
        let color = par1.into_color();
        let par2 = params[1].get_value(stack);
        let r = par2.into_rectangle();
        let par3 = params[2].get_value_mut(stack);
        let VariableValue::Image(img) = par3 else { panic!() };
        img.draw_rect((r.top_left.x as usize,r.top_left.y as usize), (r.bot_right.x as usize, r.bot_right.y as usize), color);
        None
    }

    pub fn save_as(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("save image as", params, 2);
        let par1 = params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let img = par1.into_image();
        let name = par2.into_string();
        if let Err(e) = img.save(name) {
            eprintln!("warning: could not save image as {name}: {e}");
        }
        None
    }

    pub fn load_from(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("load image", params, 1);
        let par1 = &params[0].get_value(stack);
        let name = par1.into_string();
        match image::open(name) {
            Ok(i) => Some(VariableValue::Image(i.into_rgb8())),
            Err(e) => panic!("error: could not load image {name}: {e}"),
        }
    }

    pub fn take_from(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("take from", params, 2);
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let rect = par1.into_rectangle();
        let top_left = rect.top_left;
        let mut bot_right = rect.bot_right;
        let in_img = par2.into_image();
        let width = in_img.width() as i32;
        let height = in_img.height() as i32;
        let default_color = Color::from([0,0,0]); // default to black
        if bot_right.x < top_left.x {
            bot_right.x += width;
        }
        if bot_right.y < top_left.y {
            bot_right.y += height;
        }
        let mut out_img = image::RgbImage::new((bot_right.x-top_left.x) as u32, (bot_right.y-top_left.y) as u32);
        for row in top_left.x..bot_right.x {
            for col in top_left.y..bot_right.y {
                let color = if row < 0 || row >= width || col < 0 || col >= height {
                    &default_color
                } else {
                    in_img.get_pixel(row as u32, col as u32)
                };
                out_img.put_pixel((row - top_left.x) as u32, (col - top_left.y) as u32, *color);
            }
        }
        Some(VariableValue::Image(out_img))
    }

    pub fn colored(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("colored image", params, 3);
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let par3 = &params[2].get_value(stack);
        let col = par1.into_color();
        let width = par2.into_int();
        let height = par3.into_int();
        if width < 0 {
            panic!("error: negative image width: {width}") // TODO: user friendlify
        }
        if height < 0 {
            panic!("error: negative image height {height}") // TODO: user friendlify
        }
        let mut img = image::RgbImage::new(width as u32, height as u32);
        for p in img.pixels_mut() {
            *p = col;
        }
        Some(VariableValue::Image(img))
    }
}

pub mod rectangle {
    use crate::variable::Rectangle;

    use super::*;

    pub fn new(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("new rectangle", params, 2);
        let top_left  = params[0].get_value(stack).into_pos();
        let bot_right = params[1].get_value(stack).into_pos();
        Some(VariableValue::Rectangle(Rectangle::new(top_left, bot_right)))
    }

    pub fn draw(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("draw struct rectangle", params, 2);
        if context.is_empty() { return None; }
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let c = par1.into_color();
        let r = par2.into_rectangle();
        let top_left = r.top_left;
        let bot_right = r.bot_right;
        let frame = context.get_current_frame_mut();
        frame.draw_rect((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
        None
    }

    pub fn draw_outline(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("draw rectangle outline", params, 2);
        if context.is_empty() { return None; }
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let c = par1.into_color();
        let r = par2.into_rectangle();
        let top_left = r.top_left;
        let bot_right = r.bot_right;
        let frame = context.get_current_frame_mut();
        frame.draw_rect_outline((top_left.x as usize,top_left.y as usize), (bot_right.x as usize,bot_right.y as usize), c);
        None
    }

    pub fn expand(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("expand struct rectangle", params, 2);
        let par2 = &params[1].get_value(stack);
        let step = par2.into_int();
        let par1 = &mut params[0].get_value_mut(stack);
        let r = par1.into_rectangle_mut();
        r.top_left.x = r.top_left.x.saturating_sub(step);
        r.top_left.y = r.top_left.y.saturating_sub(step);
        r.bot_right.x = r.bot_right.x.saturating_add(step);
        r.bot_right.y = r.bot_right.y.saturating_add(step);
        None
    }

    pub fn get_corner(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("get corner", params, 1);
        let r = params[0].get_value(stack).into_rectangle();
        Some(VariableValue::Pos(r.top_left))
    }

    pub fn move_by(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("move rectangle", params, 2);
        let par2 = &params[1].get_value(stack);
        let diff = par2.into_pos();
        let r = params[0].get_value_mut(stack).into_rectangle_mut();
        r.top_left.x = r.top_left.x.saturating_add(diff.x);
        r.top_left.y = r.top_left.y.saturating_add(diff.y);
        r.bot_right.x = r.bot_right.x.saturating_add(diff.x);
        r.bot_right.y = r.bot_right.y.saturating_add(diff.y);
        None
    }
}

pub mod column {
    use crate::{variable::Column, video::Extendable};

    use super::*;

    pub fn take(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("take column", params, 1);
        let at = params[0].get_value(stack).into_int();
        let frame = context.get_current_frame();
        if at < 0 {
            panic!("error: attempted to take negative column")
        }
        let col = Column::take(frame, at as u32);
        Some(VariableValue::Column(col))
    }

    pub fn append(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("append column", params, 2);
        let col = params[0].get_value(stack).into_column().clone();
        let img = params[1].get_value_mut(stack).into_image_mut();
        img.append_column(col.get());
        None
    }

    pub fn prepend(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("prepend column", params, 2);
        let col = params[0].get_value(stack).into_column().clone();
        let img = params[1].get_value_mut(stack).into_image_mut();
        img.prepend_column(col.get());
        None
    }
}

pub mod row {
    use crate::{variable::Row, video::Extendable};

    use super::*;

    pub fn take(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("take column", params, 1);
        let at = params[0].get_value(stack).into_int();
        let frame = context.get_current_frame();
        if at < 0 {
            panic!("error: attempted to take negative column")
        }
        let row = Row::take(frame, at as u32);
        Some(VariableValue::Row(row))
    }

    pub fn append(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("append column", params, 2);
        let row = params[0].get_value(stack).into_row().clone();
        let img = params[1].get_value_mut(stack).into_image_mut();
        img.append_row(row.get());
        None
    }

    pub fn prepend(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_handles: &mut Vec<ActionHandle>) -> Option<VariableValue> {
        expect_param_count("prepend column", params, 2);
        let row = params[0].get_value(stack).into_row().clone();
        let img = params[1].get_value_mut(stack).into_image_mut();
        img.prepend_row(row.get());
        None
    }
}
