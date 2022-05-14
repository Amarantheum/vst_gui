extern crate glfw;
extern crate gl;
extern crate vst_log;

use vst::editor::Editor;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use std::thread::{JoinHandle, spawn};

use parking_lot::Mutex;
use std::sync::Arc;

use glfw::{Action, Context, Key, Window, WindowEvent, Glfw, WindowHint};
use winapi::um::winuser::SetParent;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winapi::shared::windef::HWND;

pub mod gui_elements;

pub struct VstEditor
{
    size: (u32, u32),
    position: (i32, i32),
    window: Option<Window>,
    events: Option<Receiver<(f64, WindowEvent)>>,
    glfw: Glfw,
    color: [f32; 3],
    is_open: bool,
}

impl VstEditor {
    pub fn new(size: (u32, u32), position: (i32, i32), color: [f32; 3]) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(WindowHint::Decorated(false));
        glfw.window_hint(WindowHint::Decorated(false));
        glfw.window_hint(WindowHint::Visible(false));

        
        Self {
            size,
            position,
            window: None,
            events: None,
            glfw: glfw,
            color,
            is_open: false,
        }
    }
}

impl Editor for VstEditor {
    fn size(&self) -> (i32, i32) {
        (self.size.0 as i32, self.size.1 as i32)
    }
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }
    fn open(&mut self, parent: *mut c_void) -> bool {
        let (mut window, events) = self.glfw.create_window(self.size.0, self.size.1, "", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.set_pos(0, 0);
        window.make_current();
        
        window.set_key_polling(true);
        gl::load_with(|s| window.get_proc_address(s) as *const _);
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        let window_handle = match window.raw_window_handle() {
            RawWindowHandle::Win32(h) => {
                h.hwnd
            }
            _ => unimplemented!("Unimplemented"),
        };
        unsafe { SetParent(window_handle as HWND, parent as HWND); }
        
        window.show();
        self.window = Some(window);
        self.events = Some(events);
        self.is_open = true;
        
        true
    }
    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        if self.is_open() {
            let window = self.window.take().unwrap();
            window.close();
            self.events = None;
            self.is_open = false;
        }
        
    }

    fn idle(&mut self) {
        if self.is_open() {
            let window = self.window.as_mut().unwrap();
            let events = self.events.as_mut().unwrap();
            let glfw = &mut self.glfw;
    
            // Poll for and process events
            glfw.poll_events();
            /*for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::A, _, Action::Press, _) => {
                        unsafe {
                            gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                            window.swap_buffers();
                            gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                        }
                    },
                    glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                        unsafe {
                            gl::ClearColor(0.2, 0.0, 0.2, 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                            window.swap_buffers();
                            gl::ClearColor(0.2, 0.0, 0.2, 1.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);
                        }
                    },
                    _ => {},
                }
            }*/
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            
            let mut text = match gui_elements::text::UIText::new(
                "fortnite battle pass", 
                50.0, 
                [1.0, 1.0, 1.0, 1.0], 
                [100.0,100.0], 
                glyph_brush::ab_glyph::FontRef::try_from_slice(include_bytes!("gui_elements/text/fonts/source-code-pro.regular.ttf")).unwrap()
            ) {
                Ok(v) => v,
                Err(e) => {
                    vst_log::log(e.to_string());
                    panic!("bruh");
                }
            };
            text.render((640, 320));
            window.swap_buffers();
        }
        
    }
}