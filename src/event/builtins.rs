use crate::action::ActionHandle;
use crate::context;
use crate::context::Context;
use crate::variable::Variable;
use crate::variable::{Direction, VariableValue};
use crate::video::Drawable;

/// Function callable both at compiletime and runtime.
/// Such function can most notibly access the current stack of the program.
pub type Builtin = fn(&mut dyn context::Context, &mut Vec<Variable>) -> Option<VariableValue>;

/// Function callable only at runtime.
/// This allows the function to access the current frame or change the flow of program with Handles.
pub type BuiltinRuntime = fn(&mut context::Runtime, &mut Vec<Variable>) -> Option<VariableValue>;

/// Function callable only at compiletime.
/// Such function can access global options of the whole program.
pub type BuiltinCompiletime = fn(&mut context::Compiletime, &mut Vec<Variable>) -> Option<VariableValue>;

pub fn expect_param_count(operation_name: &str, params: &Vec<Variable>, expected: usize) {
    assert_eq!(params.len(), expected, "error: function {operation_name} expected {expected} parameters, got {}", params.len());
}

// both runtime and compiletime

pub fn print(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("print", params, 1);
    let par1 = context.get_value(&params[0]);
    let s = par1.into_string();
    println!("{s}");
    None
}

pub fn debug(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("debug", params, 1);
    let param = &params[0];
    let name = param.get_name();
    let val = context.get_value(param);
    println!("{name}: {val}");
    None
}

pub fn add_to(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("add", params, 2);
    let i1 = context.get_value(&params[0]).into_int();
    let i2 = context.get_value_mut(&mut params[1]).into_int_mut();
    *i2 = i2.saturating_add(i1);
    None
}

pub fn plus(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("plus", params, 2);
    let i1 = context.get_value(&params[0]).into_int();
    let i2 = context.get_value(&params[1]).into_int();
    Some(VariableValue::Int(i1.saturating_add(i2)))
}

pub fn sub(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("sub", params, 2);
    let i1 = context.get_value(&params[0]).into_int();
    let i2 = context.get_value_mut(&mut params[1]).into_int_mut();
    *i2 = i2.saturating_sub(i1);
    None
}

pub fn minus(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("minus", params, 2);
    let i1 = context.get_value(&params[0]).into_int();
    let i2 = context.get_value(&params[1]).into_int();
    Some(VariableValue::Int(i1.saturating_sub(i2)))
}

pub fn set(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("set", params, 2);
    let v2 = &params[1];
    let new_val = context.get_value(v2).clone();
    let v1 = &mut params[0];
    context.set_value(v1, new_val);
    None
}

pub fn top_into(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("top into", params, 2);
    let par1 = context.get_value(&params[0]);
    let v = par1.into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    let top = context.get_value(&v[0]).clone();
    context.set_value(&mut params[1], top);
    None
}

pub fn top(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("top", params, 1);
    let v = context.get_value(&params[0]).into_vec();
    if v.is_empty() { panic!("error: empty vector"); }
    Some(context.get_value(&v[0]).clone())
}

pub fn rotate_vec(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("rotate", params, 3);
    let d = context.get_value(&params[1]).into_direction();
    let step = context.get_value(&params[2]).into_int();
    let v = context.get_value_mut(&mut params[0]).into_vec_mut();
    match d {
        Direction::Left => {
            v.rotate_left(step as usize);
        }
        Direction::Right => {
            v.rotate_right(step as usize);
        }
        _ => {
            panic!("error: rotate vec: vector can only be rotated left or right");
        }
    }
    None
}

// pub fn move_pos_phase(context: &mut context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
//     expect_param_count("move", params, 3);
//     if context.is_empty() { return None; } // TODO: what to do with wrapping in preprocessing?
//     let par1 = context.get_value(&params[0]);
//     let par2 = context.get_value(&params[1]);
//     let par3 = context.get_value(&params[2]);
//     let mut pos = par1.into_pos();
//     let d = par2.into_direction();
//     let step = par3.into_int();
//     let width = context.get_width() as i32;
//     let height = context.get_height() as i32;
//     match d {
//         Direction::Left => {
//             pos.x = (pos.x-step).rem_euclid(width); 
//         }
//         Direction::Right => { 
//             pos.x = (pos.x+step).rem_euclid(width); 
//         }
//         Direction::Down => { 
//             pos.y = (pos.y+step).rem_euclid(height); 
//         }
//         Direction::Up => {
//             pos.y = (pos.y-step).rem_euclid(height); 
//         }
//     }
//     context.set_value(&params[0], VariableValue::Pos(pos));
//     None
// }

pub fn get_value(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    Some(context.get_value(&params[0]).clone())
}

// pub fn move_pos(context: &mut context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
//     expect_param_count("restricted move", params, 3);
//     if context.is_empty() { return None; }
//     let par1 = context.get_value(&params[0]);
//     let par2 = context.get_value(&params[1]);
//     let par3 = context.get_value(&params[2]);
//     let mut pos = par1.into_pos();
//     let d = par2.into_direction();
//     let step = par3.into_int();
//     let width = context.get_width() as i32;
//     let height = context.get_height() as i32;
//     match d {
//         Direction::Left =>
//             pos.x = (pos.x.saturating_sub(step)).max(width),
//         Direction::Right =>
//             pos.x = (pos.x.saturating_add(step)).min(width),
//         Direction::Down =>
//             pos.y = (pos.y.saturating_add(step)).min(height),
//         Direction::Up =>
//             pos.y = (pos.y.saturating_sub(step)).max(height),
//     }
//     context.set_value(&params[0], VariableValue::Pos(pos));
//     None
// }

pub fn move_by(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("move by", params, 2);
    let diff = context.get_value(&params[1]).into_pos();
    let pos = context.get_value_mut(&mut params[0]).into_pos_mut();
    pos.move_by(&diff);
    None
}

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

    pub fn draw_into(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("draw into image", params, 3);
        let par1 = context.get_value(&params[0]);
        let color = par1.into_color();
        let par2 = context.get_value(&params[1]);
        let r = par2.into_rectangle();
        let img = context.get_value_mut(&mut params[2]).into_image_mut();
        img.draw_rect((r.top_left.x as usize,r.top_left.y as usize), (r.bot_right.x as usize, r.bot_right.y as usize), color);
        None
    }

    pub fn save_as(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("save image as", params, 2);
        let img = context.get_value(&params[0]).into_image();
        let name = context.get_value(&params[1]).into_string();
        if let Err(e) = img.save(name) {
            eprintln!("warning: could not save image as {name}: {e}");
        }
        None
    }

    pub fn load_from(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("load image", params, 1);
        let name = context.get_value(&params[0]).into_string();
        match image::open(name) {
            Ok(i) => Some(VariableValue::Image(i.into_rgb8())),
            Err(e) => panic!("error: could not load image {name}: {e}"),
        }
    }

    pub fn take_from(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("take from", params, 2);
        let par1 = context.get_value(&params[0]);
        let par2 = context.get_value(&params[1]);
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

    pub fn colored(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("colored image", params, 3);
        let col = context.get_value(&params[0]).into_color();
        let width = context.get_value(&params[1]).into_int();
        let height = context.get_value(&params[2]).into_int();
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

    pub fn new(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("new rectangle", params, 2);
        let top_left  = context.get_value(&params[0]).into_pos();
        let bot_right = context.get_value(&params[1]).into_pos();
        Some(VariableValue::Rectangle(Rectangle::new(top_left, bot_right)))
    }

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

    pub fn expand(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("expand struct rectangle", params, 2);
        let par2 = context.get_value(&params[1]);
        let step = par2.into_int();
        let r = context.get_value_mut(&mut params[0]).into_rectangle_mut();
        r.top_left.x = r.top_left.x.saturating_sub(step);
        r.top_left.y = r.top_left.y.saturating_sub(step);
        r.bot_right.x = r.bot_right.x.saturating_add(step);
        r.bot_right.y = r.bot_right.y.saturating_add(step);
        None
    }

    pub fn get_corner(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("get corner", params, 1);
        let r = context.get_value(&params[0]).into_rectangle();
        Some(VariableValue::Pos(r.top_left))
    }

    pub fn move_by(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("move rectangle", params, 2);
        let par2 = context.get_value(&params[1]);
        let diff = par2.into_pos();
        let r = context.get_value_mut(&mut params[0]).into_rectangle_mut();
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

    pub fn append(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("append column", params, 2);
        let col = context.get_value(&params[0]).into_column().clone();
        let img = context.get_value_mut(&mut params[1]).into_image_mut();
        img.append_column(col.get());
        None
    }

    pub fn prepend(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("prepend column", params, 2);
        let col = context.get_value(&params[0]).into_column().clone();
        let img = context.get_value_mut(&mut params[1]).into_image_mut();
        img.prepend_column(col.get());
        None
    }
}

pub mod row {
    use crate::{variable::Row, video::Extendable};

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

    pub fn append(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("append column", params, 2);
        let row = context.get_value(&params[0]).into_row().clone();
        let img = context.get_value_mut(&mut params[1]).into_image_mut();
        img.append_row(row.get());
        None
    }

    pub fn prepend(context: &mut dyn context::Context, params: &mut Vec<Variable>) -> Option<VariableValue> {
        expect_param_count("prepend column", params, 2);
        let row = context.get_value(&params[0]).into_row().clone();
        let img = context.get_value_mut(&mut params[1]).into_image_mut();
        img.prepend_row(row.get());
        None
    }
}

// runtime

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

