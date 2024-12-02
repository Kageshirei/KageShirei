//! This module contains the function to compute the signature of a checkin instance

use kageshirei_communication_protocol::communication::Checkin;
use kageshirei_crypt::{
    encoder::{
        base64::{Encoder, Variant},
        Encoder as _,
    },
    sha3::{Digest as _, Sha3_512},
    CryptError,
};

/// Compute the signature of a checkin instance
#[expect(
    clippy::module_name_repetitions,
    reason = "The module name is correct and the function should be quickly identifiable via its name"
)]
pub fn make_signature(checkin: &Checkin) -> Result<String, CryptError> {
    let mut hasher = Sha3_512::new();
    hasher.update(checkin.operative_system.as_bytes());
    hasher.update(checkin.hostname.as_bytes());
    hasher.update(checkin.domain.as_bytes());
    hasher.update(checkin.username.as_bytes());
    hasher.update(
        serde_json::to_string(&checkin.network_interfaces)
            .unwrap()
            .as_bytes(),
    );
    hasher.update(checkin.pid.to_le_bytes());
    hasher.update(checkin.ppid.to_le_bytes());
    hasher.update(checkin.process_name.as_bytes());
    hasher.update(checkin.integrity_level.to_le_bytes());
    hasher.update(checkin.cwd.as_bytes());

    let hash = hasher.finalize();
    let hash = hash.to_vec();

    Encoder::new(Variant::Standard).encode(hash.as_slice())
}

#[cfg(test)]
mod test {
    use kageshirei_communication_protocol::{communication::Checkin, NetworkInterface};

    use super::*;

    #[test]
    fn test_make_signature_valid_input() {
        // Mock a valid Checkin object
        let checkin = Checkin {
            operative_system:   "Windows 10".to_owned(),
            hostname:           "test-host".to_owned(),
            domain:             "test-domain".to_owned(),
            username:           "test-user".to_owned(),
            network_interfaces: vec![
                NetworkInterface {
                    name:        Some("Ethernet".to_owned()),
                    dhcp_server: Some("192.168.1.1".to_owned()),
                    address:     Some("192.168.0.1".to_owned()),
                },
                NetworkInterface {
                    name:        Some("Ethernet".to_owned()),
                    dhcp_server: Some("192.168.10.1".to_owned()),
                    address:     Some("192.168.10.1".to_owned()),
                },
            ],
            pid:                12345,
            ppid:               67890,
            process_name:       "test-process.exe".to_string(),
            integrity_level:    2,
            cwd:                "C:\\Users\\test-user".to_string(),
            metadata:           None,
        };

        // Compute the signature
        let signature = make_signature(&checkin);

        // Ensure the signature is valid
        assert!(signature.is_ok());

        // Validate the signature's length
        // SHA3-512 output is 64 bytes; base64 encoding adds approximately 33% overhead
        let signature = signature.unwrap();
        assert_eq!(signature.len(), 88); // Base64 of 64 bytes is 88 characters
    }

    #[test]
    fn test_make_signature_empty_checkin() {
        // Mock an empty Checkin object
        let checkin = Checkin {
            operative_system:   "".to_string(),
            hostname:           "".to_string(),
            domain:             "".to_string(),
            username:           "".to_string(),
            network_interfaces: vec![],
            pid:                0,
            ppid:               0,
            process_name:       "".to_string(),
            integrity_level:    0,
            cwd:                "".to_string(),
            metadata:           None,
        };

        // Compute the signature
        let signature = make_signature(&checkin);

        // Ensure the signature is valid
        assert!(signature.is_ok());

        // Validate the signature's length
        let signature = signature.unwrap();
        assert_eq!(signature.len(), 88); // Even with empty data, the hash output remains consistent
    }
}
