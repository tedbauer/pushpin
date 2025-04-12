use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

/// Starts watching the specified directory for file changes and executes
/// the provided callback function when changes are detected.
///
/// # Arguments
/// * `watch_path` - The path to watch for changes
/// * `callback` - Function to call when changes are detected
/// * `recursive` - Whether to watch subdirectories recursively
///
/// # Returns
/// A join handle to the watcher thread
pub fn start_file_watcher<P, F>(
    watch_path: P,
    callback: F,
    recursive: bool,
) -> thread::JoinHandle<()>
where
    P: AsRef<Path> + Send + 'static,
    F: Fn(&Event) + Send + 'static,
{
    let path = watch_path.as_ref().to_owned();
    let recursive_mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    thread::spawn(move || {
        // Create a channel to receive the events
        let (sender, receiver) = channel();

        // Create a watcher object
        let mut watcher = match notify::recommended_watcher(move |res| match res {
            Ok(event) => sender
                .send(event)
                .unwrap_or_else(|e| eprintln!("Error sending event: {:?}", e)),
            Err(e) => eprintln!("Error watching: {:?}", e),
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error creating watcher: {:?}", e);
                return;
            }
        };

        // Add a path to be watched
        match watcher.watch(&path, recursive_mode) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error watching path: {:?}", e);
                return;
            }
        }

        // Loop forever to handle events
        loop {
            match receiver.recv() {
                Ok(event) => {
                    for page in &event.paths {
                        println!("🔄 Re-rendering site after update for page: {:?}", page);
                    }
                    callback(&event);
                }
                Err(e) => {
                    eprintln!("😥 internal error: {:?}. Please file a bug at https://github.com/tedbauer/pushpin/issues.", e);
                    break;
                }
            }
        }
    })
}
