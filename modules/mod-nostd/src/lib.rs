#![no_std]
//! # mod-nostd
//!
//! This crate provides essential utilities for `no_std` environments, including threading and
//! message-passing capabilities. It leverages low-level Windows APIs to implement complex types
//! such as threads (`NoStdThread`) and multiple-producer, single-consumer (MPSC) channels
//! (`nostd_mpsc`), enabling concurrency in `no_std` applications.
//!
//! ## Features
//! - **Threading with `NoStdThread`:** A minimal abstraction over Windows threading APIs, providing
//!   the ability to spawn and join threads in `no_std` environments.
//! - **Message Passing with `nostd_mpsc`:** A lightweight, fixed-size MPSC channel for inter-thread
//!   communication, designed to work efficiently in constrained environments.
//!
//! ## Modules
//! - [`nostd_thread`]: Provides the `NoStdThread` struct for thread creation and management.
//! - [`nostd_mpsc`]: Implements a multiple-producer, single-consumer channel for inter-thread
//!   communication.
//!
//! ## Examples
//!
//! ### Creating and Managing Threads
//! ```rust
//! use mod_win32::nt_time::delay;
//! use nostd_thread::NoStdThread;
//!
//! let my_thread = NoStdThread::spawn(move || {
//!     for i in 0 .. 5 {
//!         delay(1); // Simulate some work
//!                   // Perform a task
//!     }
//! });
//!
//! my_thread.join().expect("Thread failed to complete");
//! ```
//!
//! ### Sending and Receiving Messages
//! ```rust
//! use mod_win32::nt_time::delay;
//! use nostd_mpsc::{channel, Receiver, Sender};
//! use nostd_thread::NoStdThread;
//!
//! let (sender, receiver): (Sender<i32>, Receiver<i32>) = channel();
//!
//! // Sender thread
//! let send_thread = NoStdThread::spawn(move || {
//!     for i in 1 ..= 5 {
//!         sender.send(i).expect("Failed to send message");
//!         delay(1); // Simulate work
//!     }
//! });
//!
//! // Receiver thread
//! let receive_thread = NoStdThread::spawn(move || {
//!     for received in receiver {
//!         // Process the received value
//!     }
//! });
//!
//! send_thread.join().expect("Sender thread failed");
//! receive_thread.join().expect("Receiver thread failed");
//! ```
//!
//! ## Safety
//! The crate interacts directly with low-level Windows APIs and includes unsafe operations such as:
//! - Raw pointer manipulations
//! - Interactions with system resources and synchronization mechanisms
//!
//! ## Testing
//! The crate includes comprehensive tests for both threading and MPSC channel functionalities.
pub mod nostd_mpsc;
pub mod nostd_thread;

extern crate alloc;

#[cfg(test)]
mod tests {

    use libc_print::libc_println;
    use mod_win32::nt_time::delay;
    use nostd_mpsc::{channel, Receiver, Sender};
    use nostd_thread::NoStdThread;

    use super::*;

    #[test]
    fn test_thread() {
        let my_thread = NoStdThread::spawn(move || {
            libc_println!("Thread is running!");

            for i in 0 .. 10 {
                delay(2);
                libc_println!("For loop: {}", i);
            }

            // delay(15);
        });

        my_thread
            .unwrap()
            .join()
            .expect("Thread did not complete successfully");
    }

    #[test]
    fn test_channel() {
        // Create a new channel with a sender and receiver.
        let (sender, receiver): (Sender<i32>, Receiver<i32>) = channel();

        // Spawn a thread that will send 10 numbers to the receiver.
        let send_thread = NoStdThread::spawn(move || {
            for i in 1 ..= 10 {
                sender.send(i).expect("Failed to send");
                libc_println!("Sent: {}", i);
                delay(1); // Simulate some work before sending the next value.
            }
        });

        // Main thread will receive and print the numbers.
        let receive_thread = NoStdThread::spawn(move || {
            for received in receiver {
                libc_println!("Received: {}", received);
                delay(2); // Simulate some work after receiving a value.
            }
        });

        // Wait for both threads to complete.
        send_thread.unwrap().join().expect("Sender thread failed");
        receive_thread
            .unwrap()
            .join()
            .expect("Receiver thread failed");
    }
}
