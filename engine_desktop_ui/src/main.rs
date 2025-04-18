use engine::start_server_async;

/* For headless */
use std::thread;
use std::time::Duration;

/* For a headless server -- launch the UI from the browser by visiting http://localhost:8080 */
fn main() {
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(start_server_async());
    });

    // Block forever or until you close manually
    println!("Engine started. Press Ctrl+C to exit.");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
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
