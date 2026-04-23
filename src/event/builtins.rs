use std::collections::HashMap;
use crate::context::Context;
use crate::variable::Variable;
use crate::variable::{Stack, Direction, VariableValue};
use crate::video::Drawable;

pub type Builtin = fn(&mut Context, &mut Stack, &mut Vec<Variable>, &mut HashMap<String,bool>) -> Option<VariableValue>;

pub fn expect_param_count(operation_name: &str, params: &Vec<Variable>, expected: usize) {
    assert_eq!(params.len(), expected, "error: function {operation_name} expected {expected} parameters, got {}", params.len());
}

pub fn print(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("print", params, 1);
    let par1 = params[0].get_value(stack);
    let s = par1.into_string();
    println!("{s}");
    None
}

pub fn activate(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    set_activeness(stack, params, action_activeness, true)
}

pub fn deactivate(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    set_activeness(stack, params, action_activeness, false)
}

fn set_activeness(stack: &mut Stack, params: &mut Vec<Variable>, action_activeness: &mut HashMap<String,bool>, val: bool) -> Option<VariableValue> {
    let op_name = if val {"activate"} else {"deactivate"};
    expect_param_count(op_name, params, 1);
    // get values
    let par1 = params[0].get_value(stack);
    let label = par1.into_string();
    // perform operation
    let a = action_activeness.get_mut(label);
    let Some(ac) = a else {
        panic!("error: {op_name}: could not find action named {label}");
    };
    *ac = val;
    None
}

pub fn toggle_activeness(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    let op_name = "toggle";
    expect_param_count(op_name, params, 1);
    let par = &params[0].get_value(stack);
    let label = par.into_string();
    let Some(ac) = action_activeness.get_mut(label) else {
        panic!("error: {op_name}: could not find action named {label}");
    };
    *ac = !*ac;
    None
}

pub fn add_to(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("add", params, 2);
    let v1 = &params[0].get_value(stack);
    let v2 = &params[1].get_value(stack);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_add(i1);
    params[1].set_value(stack, VariableValue::Int(i2));
    None
}

pub fn sub(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("sub", params, 2);
    let v1 = &params[0].get_value(stack);
    let v2 = &params[1].get_value(stack);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_sub(i1);
    params[1].set_value(stack, VariableValue::Int(i2));
    None
}

pub fn set(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("set", params, 2);
    let v2 = &params[1];
    let new_val = v2.get_value(stack).clone();
    let v1 = &mut params[0];
    v1.set_value(stack, new_val);
    None
}

pub fn top_into(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("top into", params, 2);
    let par1 = &params[0].get_value(stack);
    let v = par1.into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    let top = v[0].get_value(stack).clone();
    params[1].set_value(stack, top);
    None
}

pub fn rotate_vec(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
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

pub fn get_frame(context: &mut Context, _stack: &mut Stack, _params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    println!("here");
    if context.is_empty() { return None; }
    let frame = context.get_frame();
    Some(VariableValue::Image(frame.clone()))
}

pub fn draw_rect(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let c = par1.into_color();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_rect((l as usize,t as usize), (r as usize,b as usize), c);
    None
}

pub fn draw_effect_rect(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("draw rectangle (effect)", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let e = par1.into_effect();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_effect_rect((l as usize,t as usize), (r as usize,b as usize), e);
    None
}

pub fn draw_rect_outline(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("draw rectangle", params, 3);
    if context.is_empty() { return None; }
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let par3 = &params[2].get_value(stack);
    let c = par1.into_color();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_rect_outline((l as usize,t as usize), (r as usize,b as usize), c);
    None
}

pub fn move_pos_phase(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
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
            pos.0 = (pos.0-step).rem_euclid(width); 
        }
        Direction::Right => { 
            pos.0 = (pos.0+step).rem_euclid(width); 
        }
        Direction::Down => { 
            pos.1 = (pos.1+step).rem_euclid(height); 
        }
        Direction::Up => {
            pos.1 = (pos.1-step).rem_euclid(height); 
        }
    }
    params[0].set_value(stack, VariableValue::Pos(pos.0, pos.1));
    None
}

pub fn move_pos(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
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
            pos.0 = (pos.0.saturating_sub(step)).max(width),
        Direction::Right =>
            pos.0 = (pos.0.saturating_add(step)).min(width),
        Direction::Down =>
            pos.1 = (pos.1.saturating_add(step)).min(height),
        Direction::Up =>
            pos.1 = (pos.1.saturating_sub(step)).max(height),
    }
    params[0].set_value(stack, VariableValue::Pos(pos.0, pos.1));
    None
}

pub fn move_by(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
    expect_param_count("move by", params, 2);
    let par1 = &params[0].get_value(stack);
    let par2 = &params[1].get_value(stack);
    let (mut x,mut y) = par1.into_pos();
    let (dx,dy) = par2.into_pos();
    x = x.saturating_add(dx);
    y = y.saturating_add(dy);
    params[0].set_value(stack, VariableValue::Pos(x,y));
    None
}


pub mod image {
    use super::*;
    use ::image;

    pub fn draw_at(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("draw image at", params, 2);
        if context.is_empty() { return None; }
        let par1 = params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let img = par1.into_image();
        let (x,y) = par2.into_pos();
        let frame = context.get_frame();
        image::imageops::overlay(frame, img, x.into(), y.into());
        None
    }

    pub fn draw_into(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("draw into image", params, 3);
        let par1 = params[0].get_value(stack);
        let color = par1.into_color();
        let par2 = params[1].get_value(stack);
        let VariableValue::Structure(rect) = par2 else { panic!() };
        let (l,t) = rect.get_member("0").into_pos();
        let (r,b) = rect.get_member("1").into_pos();
        let par3 = params[2].get_value_mut(stack);
        let VariableValue::Image(img) = par3 else { panic!() };
        img.draw_rect((l as usize,t as usize), (r as usize, b as usize), color);
        None
    }

    pub fn save_as(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
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

    pub fn load_from(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("load image", params, 1);
        let par1 = &params[0].get_value(stack);
        let name = par1.into_string();
        match image::open(name) {
            Ok(i) => Some(VariableValue::Image(i.into_rgb8())),
            Err(e) => panic!("error: could not load image {name}: {e}"),
        }
    }

    pub fn colored(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("colored image", params, 3);
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let par3 = &params[2].get_value(stack);
        let col = par1.into_color();
        let width = par2.into_int();
        let height = par3.into_int();
        if width <= 0 {
            panic!("error: negative image width: {width}") // TODO: user friendlify
        }
        if height <= 0 {
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
    use super::*;

    pub fn draw(context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("draw struct rectangle", params, 2);
        if context.is_empty() { return None; }
        let par1 = &params[0].get_value(stack);
        let par2 = &params[1].get_value(stack);
        let c = par1.into_color();
        let VariableValue::Structure(s) = par2 else { panic!() };
        let (l,t) = s.get_member("0").into_pos();
        let (r,b) = s.get_member("1").into_pos();
        let frame = context.get_frame();
        frame.draw_rect((l as usize,t as usize), (r as usize,b as usize), c);
        None
    }

    pub fn expand(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("expand struct rectangle", params, 2);
        let par2 = &params[1].get_value(stack);
        let step = par2.into_int();
        let par1 = &mut params[0].get_value_mut(stack);
        let VariableValue::Structure(s) = par1 else { panic!() };
        let VariableValue::Pos(l,t) = s.get_member_mut("0") else { panic!() };
        *l = l.saturating_sub(step);
        *t = t.saturating_sub(step);
        let VariableValue::Pos(r,b) = s.get_member_mut("1") else { panic!() };
        *r = r.saturating_add(step);
        *b = b.saturating_add(step);
        None
    }

    pub fn move_by(_context: &mut Context, stack: &mut Stack, params: &mut Vec<Variable>, _action_activeness: &mut HashMap<String,bool>) -> Option<VariableValue> {
        expect_param_count("move struct rectangle", params, 2);
        let par2 = &params[1].get_value(stack);
        let (x,y) = par2.into_pos();
        let par1 = &mut params[0].get_value_mut(stack);
        let VariableValue::Structure(s) = par1 else { panic!() };
        let VariableValue::Pos(l,t) = s.get_member_mut("0") else { panic!() };
        *l = l.saturating_add(x);
        *t = t.saturating_add(y);
        let VariableValue::Pos(r,b) = s.get_member_mut("1") else { panic!() };
        *r = r.saturating_add(x);
        *b = b.saturating_add(y);
        None
    }
}
