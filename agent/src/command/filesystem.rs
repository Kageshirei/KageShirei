use kageshirei_communication_protocol::communication::TaskOutput;
use mod_win32::{nt_path::change_directory, nt_peb::get_current_directory, nt_time::current_timestamp};

/// Changes the current working directory to the specified path.
///
/// This function attempts to change the directory using an internal mechanism
/// that interacts with the underlying NT API (`NtOpenFile`). Upon success, it
/// retrieves and returns the new current directory. If the operation fails, it
/// returns a failure status code, which is encapsulated in the `TaskOutput`.
///
/// # Nt API Involved
/// - `NtOpenFile`: Used internally to open the specified directory. If the operation is
///   unsuccessful, the NT status code is returned and converted to a failure result.
///
/// # Parameters
/// - `path`: A string slice representing the target directory path to switch to.
/// - `metadata`: Metadata that includes additional information to be recorded as part of the
///   command execution (e.g., timestamps, system details).
///
/// # Returns
/// - `TaskOutput`: A structure containing the details of the command execution, including:
///   - `output`: The new current directory as a `String` if the operation is successful.
///   - `exit_code`: An `Option<i32>` representing the success or failure status (0 for success,
///     non-zero for failure).
///   - `started_at` and `ended_at`: Timestamps marking the start and end of the operation.
///   - Additional metadata captured during the execution.
///
/// # Behavior
/// - If the directory change fails, an error code is stored in the `exit_code`, and the operation
///   ends.
/// - On success, the new directory is retrieved using `get_current_directory`, and the result is
///   stored in `output`.
pub fn command_cd(path: &str) -> TaskOutput {
    let mut output = TaskOutput::new();
    output.started_at = Some(current_timestamp());

    let status = change_directory(path);

    // Attempt to change the directory
    if status > 0 {
        // If the change_directory function returns a positive value, it indicates an error occurred
        output.ended_at = Some(current_timestamp());
        output.exit_code = Some(status);
        return output;
    }

    // If successful, retrieve the new current directory and return it
    let current_dir = get_current_directory();
    output.output = Some(current_dir);
    output.ended_at = Some(current_timestamp());
    output.exit_code = Some(0);
    output
}

/// Retrieves the current working directory.
///
/// This function attempts to retrieve the current directory by accessing the
/// `Process Environment Block (PEB)`, which stores environment information for the running process.
/// If the retrieval is successful, the current directory is returned as part of the `TaskOutput`.
/// If the directory cannot be retrieved, an error is recorded in the `TaskOutput`.
///
/// # Details
/// - The function reads the current directory path directly from the PEB. If it fails, the function
///   returns a failure result.
///
/// # Parameters
/// - `metadata`: Metadata that includes additional information to be recorded as part of the
///   command execution.
///
/// # Returns
/// - `TaskOutput`: A structure containing the details of the command execution, including:
///   - `output`: The current directory as a `String` if the operation is successful.
///   - `exit_code`: An `Option<i32>` representing the success or failure status (0 for success,
///     non-zero for failure).
///   - `started_at` and `ended_at`: Timestamps marking the start and end of the operation.
///   - Additional metadata captured during the execution.
///
/// # Behavior
/// - If the current directory cannot be retrieved or is empty, an error is indicated by setting the
///   `exit_code`.
/// - On success, the current directory is stored in `output`.
pub fn command_pwd() -> TaskOutput {
    let mut output = TaskOutput::new();
    output.started_at = Some(current_timestamp());

    // Retrieve the current working directory from the PEB
    let current_dir = get_current_directory();

    // Check if the current directory was successfully retrieved
    if current_dir.is_empty() {
        // If the directory is empty, an error occurred during retrieval
        output.ended_at = Some(current_timestamp());
        output.exit_code = Some(-1); // Changed to 1 to indicate an error
        return output;
    }

    // If successful, return the current directory
    output.output = Some(current_dir);
    output.ended_at = Some(current_timestamp());
    output.exit_code = Some(0); // Success
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd() {
        // Test changing to a valid directory
        let target_directory = "C:\\Windows\\System32\\drivers\\etc";

        let result = command_cd(target_directory);

        // Assert that the exit code is 0 (success)
        assert!(
            result.exit_code == Some(0),
            "Failed to change to directory: {}",
            target_directory
        );

        // Verify that the output directory matches the expected path
        assert!(
            result
                .output
                .as_ref()
                .unwrap()
                .ends_with("C:\\Windows\\System32\\drivers\\etc"),
            "Expected directory: C:\\Windows\\System32\\drivers\\etc, but got: {}",
            result.output.as_ref().unwrap()
        );

        // Test changing to the parent directory with "cd .."
        let result = command_cd("..\\..");

        // Assert that the exit code is 0 (success)
        assert!(
            result.exit_code == Some(0),
            "Failed to change to parent directory"
        );
        // Verify that the output directory matches the expected path
        assert!(
            result
                .output
                .as_ref()
                .unwrap()
                .ends_with("C:\\Windows\\System32"),
            "Expected directory: C:\\Windows\\System32, but got: {}",
            result.output.as_ref().unwrap()
        );
    }

    #[test]
    fn test_pwd() {
        let cwd_output = command_pwd();

        // Assert that the exit code is 0 (success)
        assert!(
            cwd_output.exit_code == Some(0),
            "Expected success, but exit code was: {:?}",
            cwd_output.exit_code
        );

        // Verify the output directory is not empty
        let cwd_str = cwd_output.output.as_ref().unwrap();
        assert!(
            !cwd_str.is_empty(),
            "Expected a non-empty current directory, but got an empty string"
        );
    }
}
