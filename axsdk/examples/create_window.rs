use axsdk::plugins::{*, axwindow::axwindow::AxWindow};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
        UI::HiDpi::*,
        UI::Input::*,
        UI::Input::KeyboardAndMouse::*,
        Devices::HumanInterfaceDevice::*,
        UI::TextServices::*,
        Graphics::Gdi::*,
        UI::Controls::*,
        UI::Controls::Dialogs::*
    }
};
struct App {
    window: Box<AxWindow>
}

impl App {
    pub fn new() -> Self {
        let window = AxWindow::new( "AxonEngine");

        App {
            window: window
        }
    }
    
    pub fn tick(&mut self) -> bool {
        self.window.poll_events();
        
        if self.window.has_requested_close {
            println!("App detected request to close");
            return false;
        }

        true
    }
    
    pub fn destroy(&mut self) {
        println!("App is destroying window");
        self.window.destroy();
    }   
}

fn main() {
    let mut app = App::new();

    while app.tick() {
       
    }

    unsafe {
        app.destroy();
        println!("{:?}", GetLastError());
    }
}