# SCOOBE

SCOOBE will be an application that:
- Starts minimized or hidden in the background
- On a specific trigger (like a timer or event), brings the entire window to the foreground
- Must work cross-platform

## Possible thoughts:

### Step 1: Start the window hidden/minimized

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new()
    .with_title("OOBE SDK")
    .with_inner_size(LogicalSize::new(1024.0, 768.0))
    .with_visible(false)  // Start the window hidden
    .build(&event_loop)?;

let webview = WebViewBuilder::new(&window)
    .with_url("http://localhost:8080")?
    .with_devtools(true)
    .with_initialization_script(r#"
        console.log("WebView initialized");
    "#)
    .build()?;
```

### Step 2: Create a method to bring the window to foreground

This function will bring our window to the foreground when called:

```rust
use std::sync::{Arc, Mutex};
use tao::window::Window;

// Create a function to show and focus the window
fn bring_to_foreground(window: &Window) {
    // Make window visible
    window.set_visible(true);
    
    // Platform-specific focus strategies
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowExtWindows;
        
        // Flash the taskbar icon to get user attention
        window.request_user_attention(Some(tao::window::UserAttentionType::Critical));
        
        // Temporarily set always on top to grab focus
        window.set_always_on_top(true);
        window.set_focus();
        
        // Optional: reset always-on-top after a delay
        let win_clone = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            win_clone.set_always_on_top(false);
        });
    }
    
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::WindowExtMacOS;
        
        // macOS specific methods
        window.request_user_attention(Some(tao::window::UserAttentionType::Critical));
        window.make_key_and_order_front();
        window.set_always_on_top(true);
        
        // Optional: reset always-on-top after a delay
        let win_clone = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            win_clone.set_always_on_top(false);
        });
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux approach - show the window and request attention
        window.request_user_attention(Some(tao::window::UserAttentionType::Critical));
        window.set_always_on_top(true);
        window.set_focus();
        
        // Optional: reset always-on-top after a delay
        let win_clone = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            win_clone.set_always_on_top(false);
        });
    }
}
```

### Step 3: Integrate with the event loop to handle external trigger

```rust
// Create a shared state to communicate between threads
let show_window = Arc::new(Mutex::new(false));
let show_window_clone = show_window.clone();

// Wrap in Option so we can move it out cleanly
let mut webview_opt = Some(webview);

// Example trigger - simulate a timer in another thread
// (in a real app, this could be any external event)
let window_handle = window.clone();
std::thread::spawn(move || {
    // Simulate waiting for some important event (5 seconds in this example)
    std::thread::sleep(Duration::from_secs(5));
    
    // Set the flag to show the window
    *show_window_clone.lock().unwrap() = true;
    
    // For some platforms, we can send a request right away
    bring_to_foreground(&window_handle);
});

event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
        Event::NewEvents(StartCause::Init) => {}
        Event::MainEventsCleared => {
            // Check if we need to show the window
            if *show_window.lock().unwrap() {
                bring_to_foreground(&window);
                *show_window.lock().unwrap() = false;
            }
        }
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
```

### Step 4: Add event system

```rust
use std::sync::mpsc::{channel, Receiver, TryRecvError};

// Setup a channel for communication
let (event_tx, event_rx) = channel();

// Keep a reference to the transmitter for other parts of your app
let tx_for_other_components = event_tx.clone();

// Example: In another component:
// tx_for_other_components.send("show_window").unwrap();

event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    // Check for messages to show window
    match event_rx.try_recv() {
        Ok(_) => {
            bring_to_foreground(&window);
        },
        Err(TryRecvError::Empty) => {},
        Err(TryRecvError::Disconnected) => {},
    }

    match event {
        Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
            *control_flow = ControlFlow::Exit;
            if let Some(wv) = webview_opt.take() {
                drop(wv);
            }
        }
        _ => (),
    }
});
```

This approach will work cross-platform and provides a clean way to respond to external events to show the window. The combination of platform-specific methods ensures the best chance of bringing your window to the foreground on each operating system.