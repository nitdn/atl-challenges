use iced::{
    Length::Fill,
    Task,
    widget::{self, vertical_space},
};
use pest::Parser;
use rayon::prelude::*;

use crate::{Expr, ExprParser, Rule, inorder_eval, parse_expr};

#[derive(Debug)]
pub struct Inputs {
    current_input: String,
    current_x_val: f32,
    current_result: f32,
    inputs: Vec<(String, Expr, String)>,
    is_graphing: bool,
    pixels: Vec<u8>,
    scale: f32,
    x_pan: f32,
    y_pan: f32,
}

impl Default for Inputs {
    fn default() -> Self {
        let mut pixels = vec![255u8; RESOLUTION * RESOLUTION * 4];
        paint_pixel(&mut pixels, 0.0, 0.0, 1.0, |_, _, _| false); // set up axes

        Self {
            current_input: Default::default(),
            current_x_val: Default::default(),
            inputs: Default::default(),
            is_graphing: true,
            pixels,
            scale: 1.0,
            x_pan: Default::default(),
            y_pan: Default::default(),
            current_result: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Slider(f32),
    StoreX(f32),
    EditExpression(String),
    Submit,
    ToggleGraphing(bool),
    Update(String),
    VerticalSlider(f32),
    ZoomIn,
    ZoomOut,
}
const RESOLUTION: usize = 1024;
const FONT_SIZE: u16 = 24;
fn paint_pixel<T: Fn(f32, f32, f32) -> bool + std::marker::Sync>(
    pixels: &mut [u8],
    x_pan: f32,
    y_pan: f32,
    scale: f32,
    predicate: T,
) {
    pixels
        .par_rchunks_exact_mut(RESOLUTION * 4)
        .enumerate()
        .map(|(y_coord, line)| (y_coord, line.chunks_mut(4).enumerate()))
        .for_each(|(y_coord, line)| {
            for (x_coord, pixel) in line {
                let x_center = RESOLUTION as f32 / 2.0;
                let y_center = RESOLUTION as f32 / 2.0;
                // For each increment of the pixel we actually increment x by
                // 1/(scale)
                // and the x_domain of a pixel is however
                // however far the x_center of the viewport
                // was from the origin
                // which is x_pan + (-1/2 RESOULUTION), as we want the left edge
                // of the viewport to be negative when the x_pan is zero

                let x_domain = x_pan + (-x_center + x_coord as f32) / scale;
                let y_domain = y_pan + (-y_center + y_coord as f32) / scale;
                if predicate(x_domain, y_domain, scale) {
                    pixel[0] = 200;
                    pixel[1] = 0;
                    pixel[2] = 0;
                    pixel[3] = 255;
                } else if x_domain.abs() <= 1.0 / scale || y_domain.abs() <= 1.0 / scale {
                    pixel[0] = 0;
                    pixel[1] = 0;
                    pixel[2] = 0;
                    pixel[3] = 100;
                } else {
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                    pixel[3] = 255;
                }
            }
        });
}

impl Inputs {
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        let task = match message {
            Message::Update(text) => {
                self.current_input = text;
                return Task::none();
            }
            Message::Submit => {
                if let Ok(mut pairs) = ExprParser::parse(Rule::equation, &self.current_input) {
                    let is_x = pairs
                        .clone()
                        .flatten()
                        .any(|pair| pair.as_rule() == Rule::var_x);
                    let expr = parse_expr(pairs.next().unwrap().into_inner());
                    let value = inorder_eval(&expr, self.current_x_val);

                    let input = if is_x {
                        format!(
                            "{} at x = {} is {}",
                            self.current_input, self.current_x_val, value
                        )
                    } else {
                        format!("{} = {}", self.current_input, value)
                    };
                    self.inputs.push((input, expr, self.current_input.clone()));
                    self.current_input = value.to_string();
                    self.current_result = value;
                }
                widget::text_input::focus("Text Box")
            }
            Message::Slider(slider) => {
                self.x_pan += slider / self.scale;
                Task::none()
            }
            Message::VerticalSlider(slider) => {
                self.y_pan += slider / self.scale;
                Task::none()
            }
            Message::ZoomIn => {
                self.scale *= 1.0 + 5.0 / self.scale;
                Task::none()
            }
            Message::ZoomOut => {
                self.scale /= 1.0 + 5.0 / self.scale;
                Task::none()
            }
            Message::StoreX(value) => {
                self.current_x_val = value;
                return Task::none();
            }
            Message::ToggleGraphing(is_graphing) => {
                self.is_graphing = is_graphing;
                Task::none()
            }
            Message::EditExpression(current_input) => {
                self.current_input = current_input;
                return Task::none();
            }
        };
        if self.is_graphing {
            self.render_update()
        };
        // widget::text_input::focus("Text Box")
        task
    }

    fn render_update(&mut self) {
        paint_pixel(
            &mut self.pixels,
            self.x_pan,
            self.y_pan,
            self.scale,
            |x, y, scale| {
                self.inputs.par_iter().any(|(_, expr, _)| {
                    let x_0 = x - 1.0 / scale;
                    let eval_x_0 = inorder_eval(expr, x_0);
                    let x_1 = x + 1.0 / scale;
                    let eval_x_1 = inorder_eval(expr, x_1);
                    let start = eval_x_0.min(eval_x_1);
                    let end = eval_x_0.max(eval_x_1);
                    start <= y && y <= end
                    // (start..=end).contains(&y)
                })
            },
        )
    }
    pub fn view(&self) -> iced::Element<'_, Message> {
        let items_list = self.inputs.iter().map(|item| {
            widget::row![
                widget::text(&item.0).size(FONT_SIZE),
                widget::horizontal_space(),
                widget::button(widget::text!("Edit").size(FONT_SIZE))
                    .on_press(Message::EditExpression(item.2.clone()))
            ]
            .into()
        });
        // let items_list = [];

        let scrollable_items_list =
            widget::container(widget::scrollable(widget::column(items_list))).max_height(600);
        let text_input_row = widget::container(widget::row![
            // widget::Row::from_vec(items_list),
            widget::text_input("Type an equation...", &self.current_input)
                .id("Text Box")
                .size(FONT_SIZE)
                .on_input(Message::Update)
                .on_submit(Message::Submit),
            widget::button(
                widget::text!("x = {} (Click to save last result)", self.current_x_val)
                    .size(FONT_SIZE)
            )
            .on_press(Message::StoreX(self.current_result)),
            widget::button(widget::text!("Evaluate").size(FONT_SIZE)).on_press(Message::Submit),
        ]);
        // let columns = widget::column![];

        let columns = if self.is_graphing {
            widget::column![
                widget::row![
                    widget::image(widget::image::Handle::from_rgba(
                        RESOLUTION as u32,
                        RESOLUTION as u32,
                        self.pixels.clone()
                    ))
                    .width(Fill)
                    .height(Fill),
                    widget::vertical_slider(-50.0..=50.0, 0.0, Message::VerticalSlider)
                ],
                widget::slider(-50.0..=50.0, 0.0, Message::Slider),
                widget::row![
                    widget::button(widget::text!("Zoom +").size(FONT_SIZE))
                        .on_press(Message::ZoomIn),
                    widget::button(widget::text!("Zoom -").size(FONT_SIZE))
                        .on_press(Message::ZoomOut)
                ]
            ]
        } else {
            widget::column![vertical_space()]
        }
        .push(
            widget::toggler(self.is_graphing)
                .label("Toggle Graphing")
                .text_size(FONT_SIZE)
                .on_toggle(Message::ToggleGraphing),
        )
        .height(Fill);
        widget::container(columns.push(scrollable_items_list).push(text_input_row)).into()
    }
}
