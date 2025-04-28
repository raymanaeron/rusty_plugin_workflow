use engine::start_server_async;
use std::{ thread, time::Duration };

use tao::event::{ Event, StartCause, WindowEvent };
use tao::event_loop::{ ControlFlow, EventLoop };
use tao::window::WindowBuilder;
use tao::dpi::LogicalSize;

use wry::WebViewBuilder;

// === WebSocket Event Topics ===
pub static RUST_INVOKED_EVENT: &str = "RustInvokedEvent";
pub static PLUGIN_TASK_AGENT_INVOKED_EVENT: &str = "PluginTaskAgentInvokedEvent";

fn wait_for_server() {
    use std::net::TcpStream;
    for _ in 0..20 {
        if TcpStream::connect("127.0.0.1:8080").is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    eprintln!("Warning: Server did not become available in time.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start Axum plugin engine
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });

    wait_for_server();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("OOBE SDK")
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(&window)
        .with_url("http://localhost:8080")?
        .with_devtools(true)
        .with_initialization_script(r#"
            console.log("WebView initialized");
        "#)
        .build()?;

    // Wrap in Option so we can move it out cleanly
    let mut webview_opt = Some(webview);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;

                // Take ownership and drop explicitly
                if let Some(wv) = webview_opt.take() {
                    drop(wv);
                }
            }
            _ => (),
        }
    });
}
