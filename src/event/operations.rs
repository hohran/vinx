use std::collections::HashMap;
use crate::event::*;
use crate::context::Context;
use crate::variable::values::Direction;
use crate::video::Drawable;

fn expect_param_count(operation_name: &str, params: &Vec<Variable>, expected: usize) {
    assert_eq!(params.len(), expected, "error: function {operation_name} expected {expected} parameters, got {}", params.len());
}

pub fn set_activeness(context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, params: &Vec<Variable>, val: bool) {
    let op_name = if val {"activate"} else {"deactivate"};
    expect_param_count(op_name, params, 1);
    // get values
    let par1 = params[0].get_value(context, scope);
    let label = par1.into_string();
    // perform operation
    let a = action_activeness.get_mut(label);
    let Some(ac) = a else {
        panic!("error: {op_name}: could not find action named {label}");
    };
    *ac = val;
}

pub fn toggle_activeness(context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, params: &Vec<Variable>) {
    let op_name = "toggle";
    expect_param_count(op_name, params, 1);
    let par = &params[0].get_value(context, scope);
    let label = par.into_string();
    let Some(ac) = action_activeness.get_mut(label) else {
        panic!("error: {op_name}: could not find action named {label}");
    };
    *ac = !*ac;
}

pub fn add(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("add", params, 2);
    let v1 = &params[0].get_value(context, scope);
    let v2 = &params[1].get_value(context, scope);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_add(i1);
    params[1].set_value(context, scope, VariableValue::Int(i2));
}

pub fn sub(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("sub", params, 2);
    let v1 = &params[0].get_value(context, scope);
    let v2 = &params[1].get_value(context, scope);
    let i1 = v1.into_int();
    let mut i2 = v2.into_int();
    i2 = i2.saturating_sub(i1);
    params[1].set_value(context, scope, VariableValue::Int(i2));
}

pub fn set(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("set", params, 2);
    let v2 = &params[1];
    let new_val = v2.get_value(context, scope);
    let v1 = &mut params[0];
    v1.set_value(context, scope, new_val);
}

pub fn top_into(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("top into", params, 2);
    let par1 = &params[0].get_value(context, scope);
    let v = par1.into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    let top = v[0].get_value(context, scope);
    params[1].set_value(context, scope, top);
}

pub fn rotate_vec(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("rotate", params, 3);
    let par1 = params[0].get_value(context, scope);
    let par2 = params[1].get_value(context, scope);
    let par3 = params[2].get_value(context, scope);
    let mut v = par1.into_vec().clone();
    if v.is_empty() { return; }
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
    params[0].set_value(context, scope, VariableValue::Vec(v));
}

pub fn draw_rect(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    expect_param_count("draw rectangle", params, 3);
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let par3 = &params[2].get_value(context, scope);
    let c = par1.into_color();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_rect((l as usize,t as usize), (r as usize,b as usize), c);
}

pub fn draw_effect_rect(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    expect_param_count("draw rectangle (effect)", params, 3);
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let par3 = &params[2].get_value(context, scope);
    let e = par1.into_effect();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_effect_rect((l as usize,t as usize), (r as usize,b as usize), e);
}

pub fn draw_rect_outline(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    expect_param_count("draw rectangle", params, 3);
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let par3 = &params[2].get_value(context, scope);
    let c = par1.into_color();
    let (l,t) = par2.into_pos();
    let (r,b) = par3.into_pos();
    let frame = context.get_frame();
    frame.draw_rect_outline((l as usize,t as usize), (r as usize,b as usize), c);
}

pub fn move_pos_phase(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("move", params, 3);
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let par3 = &params[2].get_value(context, scope);
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
    params[0].set_value(context, scope, VariableValue::Pos(pos.0, pos.1));
}

pub fn move_pos(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("restricted move", params, 3);
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let par3 = &params[2].get_value(context, scope);
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
    params[0].set_value(context, scope, VariableValue::Pos(pos.0, pos.1));
}

pub fn move_by(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    expect_param_count("move by", params, 2);
    let par1 = &params[0].get_value(context, scope);
    let par2 = &params[1].get_value(context, scope);
    let (mut x,mut y) = par1.into_pos();
    let (dx,dy) = par2.into_pos();
    x = x.saturating_add(dx);
    y = y.saturating_add(dy);
    params[0].set_value(context, scope, VariableValue::Pos(x,y));
}
