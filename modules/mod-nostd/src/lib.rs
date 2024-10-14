#![no_std]
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
        send_thread.join().expect("Sender thread failed");
        receive_thread.join().expect("Receiver thread failed");
    }
}
