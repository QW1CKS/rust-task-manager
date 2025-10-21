//! Background update loop for periodic metric collection

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::windows::monitor::{ProcessSnapshot, SystemMonitor};

/// Messages sent from updater thread to UI thread
#[derive(Debug)]
pub enum UpdateMessage {
    /// New process snapshot available
    Snapshot(ProcessSnapshot),
    /// Update loop encountered an error
    Error(String),
    /// Update loop is shutting down
    Shutdown,
}

/// Control messages sent to updater thread
#[derive(Debug)]
pub enum ControlMessage {
    /// Pause updates
    Pause,
    /// Resume updates
    Resume,
    /// Shutdown updater thread
    Shutdown,
}

/// Background updater managing periodic metric collection
///
/// # Threading Model
///
/// Runs in dedicated background thread, sends updates via mpsc channel.
/// UI thread receives updates without blocking.
pub struct Updater {
    /// Handle to background thread
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Channel to send control messages to updater
    control_tx: Sender<ControlMessage>,
}

impl Updater {
    /// Start background updater with specified refresh rate
    ///
    /// # Arguments
    ///
    /// * `refresh_rate_ms` - How often to collect metrics (milliseconds)
    ///
    /// # Returns
    ///
    /// (Updater, Receiver) - Updater handle and receiver for updates
    pub fn start(refresh_rate_ms: u64) -> (Self, Receiver<UpdateMessage>) {
        let (update_tx, update_rx) = channel();
        let (control_tx, control_rx) = channel();

        let thread_handle = thread::spawn(move || {
            run_update_loop(refresh_rate_ms, update_tx, control_rx);
        });

        let updater = Self {
            thread_handle: Some(thread_handle),
            control_tx,
        };

        (updater, update_rx)
    }

    /// Pause updates
    pub fn pause(&self) {
        let _ = self.control_tx.send(ControlMessage::Pause);
    }

    /// Resume updates
    pub fn resume(&self) {
        let _ = self.control_tx.send(ControlMessage::Resume);
    }

    /// Shutdown updater thread gracefully
    pub fn shutdown(&mut self) {
        let _ = self.control_tx.send(ControlMessage::Shutdown);
        
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Updater {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Main update loop running in background thread
///
/// # Performance
///
/// Maintains precise timing using sleep duration adjustment.
/// Target: 1Hz (1000ms) with <5ms jitter.
fn run_update_loop(
    refresh_rate_ms: u64,
    update_tx: Sender<UpdateMessage>,
    control_rx: Receiver<ControlMessage>,
) {
    let mut monitor = SystemMonitor::new();
    let mut paused = false;
    let refresh_duration = Duration::from_millis(refresh_rate_ms);

    loop {
        let cycle_start = Instant::now();

        // Check for control messages (non-blocking)
        if let Ok(msg) = control_rx.try_recv() {
            match msg {
                ControlMessage::Pause => paused = true,
                ControlMessage::Resume => paused = false,
                ControlMessage::Shutdown => {
                    let _ = update_tx.send(UpdateMessage::Shutdown);
                    break;
                }
            }
        }

        // Collect metrics if not paused
        if !paused {
            match monitor.collect_all() {
                Ok(snapshot) => {
                    if update_tx.send(UpdateMessage::Snapshot(snapshot)).is_err() {
                        // Receiver dropped, exit loop
                        break;
                    }
                }
                Err(e) => {
                    if update_tx.send(UpdateMessage::Error(e)).is_err() {
                        break;
                    }
                }
            }
        }

        // Sleep for remaining time to maintain precise refresh rate
        let elapsed = cycle_start.elapsed();
        if elapsed < refresh_duration {
            thread::sleep(refresh_duration - elapsed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updater_starts() {
        let (mut updater, _rx) = Updater::start(1000);
        updater.shutdown();
    }

    #[test]
    fn test_updater_sends_updates() {
        let (mut updater, rx) = Updater::start(100); // 100ms for faster test

        // Wait for first update
        let msg = rx.recv_timeout(Duration::from_secs(1));
        assert!(msg.is_ok(), "Should receive update within 1 second");

        match msg.unwrap() {
            UpdateMessage::Snapshot(snapshot) => {
                assert!(!snapshot.processes.is_empty(), "Should have processes");
            }
            _ => panic!("Expected Snapshot message"),
        }

        updater.shutdown();
    }

    #[test]
    fn test_updater_pause_resume() {
        let (mut updater, rx) = Updater::start(100);

        // Get first update
        let _ = rx.recv_timeout(Duration::from_secs(1)).unwrap();

        // Pause
        updater.pause();
        thread::sleep(Duration::from_millis(300));

        // Should not receive many updates while paused
        let mut count = 0;
        while let Ok(_) = rx.try_recv() {
            count += 1;
        }
        assert!(count < 3, "Should receive few updates while paused");

        // Resume
        updater.resume();
        let msg = rx.recv_timeout(Duration::from_secs(1));
        assert!(msg.is_ok(), "Should receive update after resume");

        updater.shutdown();
    }
}
