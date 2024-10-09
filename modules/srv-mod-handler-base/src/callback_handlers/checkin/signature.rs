use bytes::Bytes;

use rs2_communication_protocol::communication_structs::checkin::Checkin;
use rs2_crypt::encoder::base64::Base64Encoder;
use rs2_crypt::encoder::Encoder;
use rs2_crypt::sha3::{Digest, Sha3_512};

/// Compute the signature of a checkin instance
pub fn make_signature(checkin: &Checkin) -> String {
	let mut hasher = Sha3_512::new();
	hasher.update(checkin.operative_system.as_bytes());
	hasher.update(checkin.hostname.as_bytes());
	hasher.update(checkin.domain.as_bytes());
	hasher.update(checkin.username.as_bytes());
	hasher.update(checkin.ip.as_bytes());
	hasher.update(checkin.process_id.to_le_bytes());
	hasher.update(checkin.parent_process_id.to_le_bytes());
	hasher.update(checkin.process_name.as_bytes());
	hasher.update(checkin.integrity_level.to_le_bytes());
	hasher.update(checkin.cwd.as_bytes());

	let hash = hasher.finalize();
	let hash = Bytes::from(hash.to_vec());

	let encoder = Base64Encoder;
	encoder.encode(hash)
}

#[cfg(test)]
mod test {
	use rs2_communication_protocol::communication_structs::checkin::PartialCheckin;

	use super::*;

	#[test]
	fn test_make_signature() {
		let checkin = Checkin::new(PartialCheckin {
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "WORKGROUP".to_string(),
			username: "user".to_string(),
			ip: "10.2.123.45".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "agent.exe".to_string(),
			elevated: true,
		});

		let signature = make_signature(&checkin);

		println!("Signature: {}", signature);

		assert_eq!(signature, "YdkxtuNA9_78BiX7Oe_445oEr_Rktlcve1k73kBQ9pvoq_04qXVVcRfenXjy5Sc6947p9dn_YSiLGFw6YVXp0g");
	}
}