#![allow(dead_code)]
pub mod mesh;
pub mod models;

use odc_core::mdl::{RenderModel, Size2d};
use odc_core::{DrawData, DrawDataSource, OdcCore, Stage, WindowInfo};
use std::collections::HashMap;
use winit::dpi::PhysicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub trait Example {
    fn render_model() -> RenderModel;
    fn windows() -> Vec<(usize, String, Size2d)>;
    fn init(&mut self, renderer: &OdcCore);
    fn update(&mut self, renderer: &OdcCore);
    fn draw_stages(&self) -> Vec<Stage>;
    fn draw_data(&self) -> DrawDataTree;
}

pub fn run_example<E: Example + 'static>(mut ex: E) -> ! {
    env_logger::init();
    let event_loop = EventLoop::new();

    let mut renderer = OdcCore::new(E::render_model());

    let windows = E::windows();
    let window_handles: HashMap<_, _> = windows
        .iter()
        .map(|(source, name, size)| {
            let window_size = PhysicalSize::new(size.x as u32, size.y as u32);
            let window = WindowBuilder::default()
                .with_title(name)
                .with_inner_size(window_size)
                .build(&event_loop)
                .unwrap();

            let window_info = WindowInfo {
                name,
                handle: &window,
                size: *size,
            };

            unsafe { renderer.add_window(*source, window_info) };
            (window.id(), (name.clone(), window, *source))
        })
        .collect();

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Poll;
        match event {
            Event::NewEvents(cause) => match cause {
                StartCause::Init => ex.init(&renderer),
                StartCause::Poll => ex.update(&renderer),
                _ => {}
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } => {
                let size = Size2d {
                    x: size.width as _,
                    y: size.height as _,
                };

                let window = &window_handles[&window_id];
                renderer.resize_window(&window.0, size);
                renderer.resize_attachments(window.2, size);
            }
            Event::MainEventsCleared => {
                let stages = ex.draw_stages();
                let data = ex.draw_data();
                renderer.draw(&data, &stages);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                for window in &windows {
                    renderer.remove_window(&window.1)
                }
                *flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

pub struct DrawDataTree(pub Vec<Vec<Vec<DrawData>>>);

impl DrawDataSource for DrawDataTree {
    fn draw_data(&self, pass: usize, pipeline: usize) -> &[DrawData] {
        &self.0[pass][pipeline]
    }
}
