#[allow(unused_imports)]
use imgui::{BackendFlags, Context, Key};
use rot_events::Event_Base::{Event, State};
use rot_events::{
    KeyboardInput::KeyCode,
    MouseInput::{Button, TypeOfMouseEvent},
};
use rot_layer::Layer;

use rot_wgpu::Renderer;
use winit::dpi::LogicalSize;
use winit::window::Window;

pub struct Gui {
    context: Context,
    scale_factor: f64,

    //for ROT_Layer implementation
    name: String,
    index_on_stack: Option<usize>,
}

impl Gui {
    pub fn build(name: String, window: &Window) -> Self {
        let mut context = Context::create();

        let io = context.io_mut();
        io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);

        let scale_factor = window.scale_factor();
        io.display_framebuffer_scale = [scale_factor as f32, scale_factor as f32];

        let logical_size: LogicalSize<f32> = window.inner_size().to_logical(scale_factor);
        io.display_size = [logical_size.width as f32, logical_size.height as f32];

        Gui {
            context,
            scale_factor,
            name,
            index_on_stack: None,
        }
    }

    pub fn attach_window(&mut self, _window: &Window) {}
}

impl Layer for Gui {
    fn on_attach(&mut self, renderer: &Renderer) {}

    fn on_event(&mut self, event: &Event) {
        let io = self.context.io_mut();
        match event {
            Event::MouseInput(ev) => match ev.ev_type {
                TypeOfMouseEvent::Button => {
                    let button = ev.mouse_button.as_ref().unwrap();
                    let pressed = button.state == State::Pressed;
                    match button.button {
                        Button::Left => io.mouse_down[0] = pressed,
                        Button::Right => io.mouse_down[1] = pressed,
                        Button::Middle => io.mouse_down[2] = pressed,
                        Button::Other(a) => io.mouse_down[a as usize] = pressed,
                    }
                }
                TypeOfMouseEvent::Wheel => {
                    let wheel = ev.mouse_wheel.as_ref().unwrap();
                    io.mouse_wheel = wheel.line_delta.y as f32;
                    io.mouse_wheel = wheel.line_delta.x as f32
                }
                TypeOfMouseEvent::Movement => {
                    let movement = ev.mouse_movement.as_ref().unwrap();
                    let x = movement.position.x / self.scale_factor;
                    let y = movement.position.y / self.scale_factor;
                    io.mouse_pos = [x as f32, y as f32];
                }
            },
            Event::KeyboardInput(ev) => {
                let pressed = ev.state == State::Pressed;
                let key = *ev.virtual_keycode.as_ref().unwrap();
                io.keys_down[key as usize] = pressed;

                match key {
                    KeyCode::LShift | KeyCode::RShift => io.key_shift = pressed,
                    KeyCode::LControl | KeyCode::RControl => io.key_ctrl = pressed,
                    KeyCode::LAlt | KeyCode::RAlt => io.key_alt = pressed,
                    KeyCode::LWin | KeyCode::RWin => io.key_super = pressed,
                    _ => {}
                }
            }
        }
    }

    fn on_update(&mut self, renderer: &mut Renderer, delta_time: f64) {}

    fn get_name(&self) -> &String {
        &self.name
    }
}
