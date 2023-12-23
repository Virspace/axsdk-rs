use axsdk::plugins::{*, axwindow::axwindow::AxWindow};

fn main() {
    let mut _window = AxWindow::new( "AxonEngine");

    let mut should_close = false;
    while !should_close {
        _window.poll_events();

        if _window.has_requested_close {
            should_close = true;
            println!("Window has requested close");
        }
    }
}
