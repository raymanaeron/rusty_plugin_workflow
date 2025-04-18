use engine::start_server_async;
use std::{thread, time::Duration};

use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;

use wry::{WebView, WebViewBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the Axum engine
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });

    thread::sleep(Duration::from_secs(1));

    // Set up native window + event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Device Setup")
        .build(&event_loop)?;

    let webview_builder = WebViewBuilder::new(&window)
        .with_url("http://localhost:8080")?; 

    // ⚠️ FIX 3: Call .build() to get the actual WebView
    let _webview: WebView = webview_builder.build()?;

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}




/* For a UI that hosts a WebView window */
/*
fn main() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });

    // Then launch WebView window to http://localhost:8080
}
*/
