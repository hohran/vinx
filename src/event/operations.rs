use std::collections::HashMap;
use crate::event::*;
use crate::context::Context;
use crate::variable::values::Direction;
use crate::video::Drawable;


pub fn set_activeness(context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, params: &Vec<Variable>, val: bool) {
    assert!(params.len() == 1, "expected 1 parameter, got {}", params.len());
    let label = &params[0];
    let label_val = label.get_value(context, scope);
    if let VariableValue::String(l) = label_val {
        let b = action_activeness.get_mut(&l).expect(&format!("error: unknown action {l}"));
        *b = val;
    } else {
        panic!("error: unexpected value type {:?}", label_val);
    }
}

pub fn toggle_activeness(context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, params: &Vec<Variable>) {
    assert!(params.len() == 1, "expected 1 parameter, got {}", params.len());
    let label = &params[0];
    let label_val = label.get_value(context, scope);
    let VariableValue::String(l) = label_val else {
        panic!("error: unexpected value type {:?}", label_val);
    };
    let b = action_activeness.get_mut(&l).expect(&format!("error: unknown action {l}"));
    *b = !*b;
}

pub fn add(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    let v1 = &params[0];
    let v2 = &params[1];
    match (v1.get_value(context, scope),v2.get_value(context, scope)) {
        (VariableValue::Int(i1),VariableValue::Int(i2)) => {
            params[1].set_value(context, scope, VariableValue::Int(i1.saturating_add(i2)));
        }
        (v1,v2) => panic!("error: unexpected operand types {:?} and {:?}",v1,v2)
    }
}

pub fn set(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    assert!(params.len() == 2, "expected 2 parameters, got {}", params.len());
    let v2 = &params[1];
    let new_val = v2.get_value(context, scope);
    let v1 = &mut params[0];
    v1.set_value(context, scope, new_val);
}

pub fn top_into(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    assert!(params.len() == 2, "expected 2 parameters, got {:?}", params);
    let v = &params[0];
    let v_val = v.get_value(context, scope);
    if let VariableValue::Vec(v) = v_val {
        if v.is_empty() { panic!("error: empty vector"); }
        let top = &v[0].get_value(context, scope);
        params[1].set_value(context, scope, top.clone());
    } else {
        panic!("error: expected v to be vector, got {:?}", v_val);
    }
}

pub fn rotate_vec(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    assert!(params.len() == 2 || params.len() == 3, "expected 2 or 3 parameters, got {}", params.len());
    let v = &params[0];
    let d = &params[1];
    let v_val = v.get_value(context, scope);
    let d_val = d.get_value(context, scope);
    let step_val = if params.len() == 2 { 1 } else { 
        let step = &params[2].get_value(context, scope);
        if let VariableValue::Int(i) = step {
            *i
        } else {
            panic!("error: expected step to be int, got {:?}", step)
        }
    };
    match (v_val,d_val) {
        (VariableValue::Vec(mut arr),VariableValue::Direction(d)) => {
            match d {
                Direction::Left => {
                    if arr.len() == 0 { return; }
                    for _ in 0..step_val {
                        let e = arr.remove(0);
                        arr.push(e);
                    }
                }
                Direction::Right => {
                    if arr.len() == 0 { return; }
                    for _ in 0..step_val {
                        if let Some(e) = arr.pop() {
                            arr.insert(0, e);
                        }
                    }
                }
                _ => panic!("error: expected only directions left and right")
            }
            params[0].set_value(context, scope, VariableValue::Vec(arr));
        }
        (v,d) => panic!("error: expected variables to be of type vector and direction, got {:?} and {:?}",v,d)
    }
}

pub fn draw_rect(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let c = &params[0];
    let tl = &params[1];
    let br = &params[2];
    let tl_val = tl.get_value(context, scope);
    let br_val = br.get_value(context, scope);
    let c_val = c.get_value(context, scope);
    let p;
    if let VariableValue::Color(c) = c_val {
        p = c.clone();
    } else {
        panic!("invalid type for color c: {:?}", c_val);
    }
    match (tl_val,br_val) {
        (VariableValue::Pos(l, t), VariableValue::Pos(r, b)) => {
            let frame = context.get_frame();
            frame.draw_rect((l,t), (r,b), p);
        }
        _ => { 
            panic!("expected tl/br to be positions");
        }
    }
}

pub fn draw_effect_rect(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let c = &params[0];
    let tl = &params[1];
    let br = &params[2];
    let tl_val = tl.get_value(context, scope);
    let br_val = br.get_value(context, scope);
    let e_val = c.get_value(context, scope);
    let VariableValue::Effect(e) = e_val else {
        panic!("invalid type for effect: {:?}", e_val);
    };
    match (tl_val,br_val) {
        (VariableValue::Pos(l, t), VariableValue::Pos(r, b)) => {
            let frame = context.get_frame();
            frame.draw_effect_rect((l,t), (r,b), e);
        }
        _ => { 
            panic!("expected tl/br to be positions");
        }
    }
}

pub fn draw_rect_outline(context: &mut Context, scope: &mut Stack, params: &Vec<Variable>) {
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let c = &params[0];
    let tl = &params[1];
    let br = &params[2];
    let tl_val = tl.get_value(context, scope);
    let br_val = br.get_value(context, scope);
    let c_val = c.get_value(context, scope);
    let p;
    if let VariableValue::Color(c) = c_val {
        p = c.clone();
    } else {
        panic!("invalid type for color c: {:?}", c_val);
    }
    match (tl_val,br_val) {
        (VariableValue::Pos(l, t), VariableValue::Pos(r, b)) => {
            let frame = context.get_frame();
            frame.draw_rect_outline((l,t), (r,b), p);
        }
        _ => { 
            panic!("expected tl/br to be positions");
        }
    }
}

pub fn move_pos_phase(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let pos = &params[0];
    let d = &params[1];
    let _step = &params[2];
    let pos_val = pos.get_value(context, scope);
    let d_val = d.get_value(context, scope);
    let step_val = _step.get_value(context, scope);
    // get step
    let step;
    if let VariableValue::Int(i) = step_val {
        step = i as usize;
    } else {
        panic!("invalid type for step: {:?} (expected Int) in variable {}", step_val, _step.get_name());
    }
    let width = context.get_width() as i32;
    let height = context.get_height() as i32;
    match pos_val {
        VariableValue::Pos(mut x, mut y) => {
            match d_val {
                VariableValue::Direction(Direction::Left) => {
                    x = (x as i32-step as i32).rem_euclid(width as i32) as usize; 
                }
                VariableValue::Direction(Direction::Right) => { 
                    x = (x as i32+step as i32).rem_euclid(width as i32) as usize; 
                }
                VariableValue::Direction(Direction::Down) => { 
                    y = (y as i32+step as i32).rem_euclid(height as i32) as usize; 
                }
                VariableValue::Direction(Direction::Up) => {
                    y = (y as i32-step as i32).rem_euclid(height as i32) as usize; 
                }
                _ => {
                    panic!("invalid type for direction: {:?}", step_val);
                }
            };
            // println!(" MOVE: {} -> ({x},{y})", pos_val.to_string());
            params[0].set_value(context, scope, VariableValue::Pos(x,y));
        }
        x => {
            panic!("unexpected type for move: {:?}", x);
        }
    }
}

pub fn move_pos(context: &mut Context, scope: &mut Stack, params: &mut Vec<Variable>) {
    assert!(params.len() == 3, "expected 3 parameters, got {}", params.len());
    let pos = &params[0];
    let d = &params[1];
    let step = &params[2];
    let pos_val = pos.get_value(context, scope);
    let d_val = d.get_value(context, scope);
    let step_val = step.get_value(context, scope);
    // get step
    let step;
    if let VariableValue::Int(i) = step_val {
        step = i as usize;
    } else {
        panic!("invalid type for step: {:?} (expected Int)", step_val);
    }
    match pos_val {
        VariableValue::Pos(mut x, mut y) => {
            match d_val {
                VariableValue::Direction(Direction::Left) => { x = x.saturating_sub(step); }
                VariableValue::Direction(Direction::Right) => { x = (x+step).min(context.get_width()); }
                VariableValue::Direction(Direction::Down) => { y = (y+step).min(context.get_height()); }
                VariableValue::Direction(Direction::Up) => { y = y.saturating_sub(step); }
                _ => {
                    panic!("invalid type for direction: {:?}", step_val);
                }
            };
            params[0].set_value(context, scope, VariableValue::Pos(x,y));
        }
        x => {
            panic!("unexpected type for move: {:?}", x);
        }
    }
}

