#[allow(unused_imports)]

mod tests {
    use libc_print::libc_println;

    use crate::command::{command_cd, command_checkin, command_exit, command_pwd};

    #[test]
    fn test_checkin() {
        // Test gathering system information and metadata for check-in
        let result = command_checkin();

        // Ensure the result is successful
        assert!(
            result.is_ok(),
            "Failed to execute check-in command: {:?}",
            result.err()
        );

        // Verify the JSON output contains expected fields
        let json_output = result.unwrap();
        assert!(json_output.contains("hostname"), "Missing 'hostname' field");
        assert!(
            json_output.contains("operative_system"),
            "Missing 'operative_system' field"
        );
        assert!(json_output.contains("ip"), "Missing 'ip' field");
        assert!(
            json_output.contains("process_id"),
            "Missing 'process_id' field"
        );
        assert!(
            json_output.contains("parent_process_id"),
            "Missing 'parent_process_id' field"
        );
        assert!(
            json_output.contains("integrity_level"),
            "Missing 'integrity_level' field"
        );

        // You can extend these checks with specific values or fields
    }

    #[test]
    fn test_cd() {
        // Test changing to a valid directory
        let target_directory = "C:\\Windows\\System32\\drivers\\etc";
        let result = command_cd(target_directory);
        assert!(
            result.is_ok(),
            "Failed to change to directory: {}",
            target_directory
        );
        let result_dir = result.unwrap();
        assert!(
            result_dir.ends_with("C:\\Windows\\System32\\drivers\\etc"),
            "Expected directory: C:\\Windows\\System32\\drivers\\etc, but got: {}",
            result_dir
        );

        // Test changing to the parent directory with "cd .."
        let result = command_cd("..\\..");
        assert!(result.is_ok(), "Failed to change to parent directory");
        let result_dir = result.unwrap();
        assert!(
            result_dir.ends_with("C:\\Windows\\System32"),
            "Expected directory: C:\\Windows, but got: {}",
            result_dir
        );
    }

    #[test]
    fn test_pwd() {
        let cwd = command_pwd();
        assert!(
            cwd.is_ok(),
            "Expected cwd to be Ok, but got an error: {:?}",
            cwd
        );

        // Optionally, you can unwrap after confirming it's Ok
        let cwd_str = cwd.unwrap();
        assert!(
            !cwd_str.is_empty(),
            "Expected a non-empty current directory"
        );
    }
}
