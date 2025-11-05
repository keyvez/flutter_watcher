use anyhow::{Context, Result};
use clap::Parser;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Flutter project directory
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Device ID to run Flutter on (optional)
    #[arg(short, long)]
    device: Option<String>,

    /// Additional Flutter run arguments
    #[arg(last = true)]
    flutter_args: Vec<String>,
}

struct FlutterProcess {
    child: Child,
}

impl FlutterProcess {
    fn spawn(path: &PathBuf, device: Option<&String>, extra_args: &[String]) -> Result<Self> {
        let mut cmd = Command::new("flutter");
        cmd.arg("run")
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if let Some(dev) = device {
            cmd.arg("-d").arg(dev);
        }

        for arg in extra_args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .context("Failed to spawn flutter run process. Make sure Flutter is installed and in PATH.")?;

        println!("🚀 Started Flutter process (PID: {})", child.id());

        Ok(FlutterProcess { child })
    }

    fn send_reload(&mut self) -> Result<()> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            stdin.write_all(b"r")?;
            stdin.flush()?;
            println!("🔄 Sent hot reload command");
            Ok(())
        } else {
            anyhow::bail!("Flutter process stdin not available")
        }
    }

    fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn kill(&mut self) -> Result<()> {
        self.child.kill().context("Failed to kill Flutter process")
    }
}

impl Drop for FlutterProcess {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

fn watch_files(path: PathBuf, process: Arc<Mutex<FlutterProcess>>) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .context("Failed to create file watcher")?;

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .context("Failed to watch directory")?;

    println!("👀 Watching for changes in: {}", path.display());
    println!("💡 Tip: Press Ctrl+C to stop\n");

    let mut last_reload = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(500);

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if should_trigger_reload(&event) {
                    // Debounce: only reload if enough time has passed
                    let now = std::time::Instant::now();
                    if now.duration_since(last_reload) >= debounce_duration {
                        // Check if process is still running
                        let mut proc = process.lock().unwrap();
                        if proc.is_running() {
                            if let Some(path) = event.paths.first() {
                                println!("📝 File changed: {}", path.display());
                            }
                            if let Err(e) = proc.send_reload() {
                                eprintln!("❌ Error sending reload: {}", e);
                            }
                            last_reload = now;
                        } else {
                            println!("❌ Flutter process has stopped");
                            break;
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("❌ Watch error: {}", e),
            Err(e) => {
                eprintln!("❌ Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn should_trigger_reload(event: &Event) -> bool {
    // Only trigger on modify, create, or remove events
    let relevant_kind = matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    );

    if !relevant_kind {
        return false;
    }

    // Only trigger for Dart files
    event.paths.iter().any(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "dart")
            .unwrap_or(false)
    })
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Verify the path exists
    if !args.path.exists() {
        anyhow::bail!("Path does not exist: {}", args.path.display());
    }

    if !args.path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", args.path.display());
    }

    // Check for pubspec.yaml to confirm it's a Flutter project
    let pubspec = args.path.join("pubspec.yaml");
    if !pubspec.exists() {
        eprintln!(
            "⚠️  Warning: pubspec.yaml not found in {}",
            args.path.display()
        );
        eprintln!("   This might not be a Flutter project root.");
    }

    println!("🎯 Flutter Watcher");
    println!("================\n");

    // Spawn Flutter process
    let process = FlutterProcess::spawn(&args.path, args.device.as_ref(), &args.flutter_args)?;
    let process = Arc::new(Mutex::new(process));

    // Setup Ctrl+C handler
    let process_clone = Arc::clone(&process);
    ctrlc::set_handler(move || {
        println!("\n🛑 Received Ctrl+C, shutting down...");
        let mut proc = process_clone.lock().unwrap();
        let _ = proc.kill();
        std::process::exit(0);
    })
    .context("Error setting Ctrl+C handler")?;

    // Wait a bit for Flutter to start up
    thread::sleep(Duration::from_secs(2));

    // Start watching for file changes
    watch_files(args.path, process)?;

    println!("👋 Flutter Watcher stopped");
    Ok(())
}
