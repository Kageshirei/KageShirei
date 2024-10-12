use std::io::Write;

use log::info;
use rcgen::{DnType, KeyPair, PKCS_ED25519};

use crate::cli::generate::certificate::GenerateCertificateArguments;

/// Create a new self-signed tls certificate for the server
///
/// # Parameters
///
/// - `args` - The arguments for the certificate generation.
///
/// # Returns
///
/// A result indicating success or failure.
pub fn make_tls(args: &GenerateCertificateArguments) -> Result<(), String> {
    info!("Generating certificate for domains: {:?}", args.domains);
    info!("Validity period: {} to {}", args.not_before, args.not_after);
    info!("Common name: {}", args.common_name);
    info!("Organization name: {}", args.organization_name);

    // Create a CA certificate
    info!("Creating CA certificate");

    let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
    ca_params
        .distinguished_name
        .push(DnType::OrganizationName, args.organization_name.as_str());
    ca_params
        .distinguished_name
        .push(DnType::CommonName, args.common_name.as_str());
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::CrlSign,
    ];
    let ca_key = KeyPair::generate_for(&PKCS_ED25519).unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    // Create a server end entity cert issued by the CA.
    info!("Creating server certificate");

    let mut server_ee_params = rcgen::CertificateParams::new(args.domains.clone()).unwrap();
    server_ee_params.is_ca = rcgen::IsCa::NoCa;
    server_ee_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];

    // Set validity period
    // Parse the not_before date and set it
    let not_before_pieces = args
        .not_before
        .split("-")
        .map(|x| x.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();
    server_ee_params.not_before = rcgen::date_time_ymd(
        not_before_pieces[0] as i32,
        not_before_pieces[1] as u8,
        not_before_pieces[2] as u8,
    );
    // Parse the not_after date and set it
    let not_after_pieces = args
        .not_after
        .split("-")
        .map(|x| x.parse::<u32>().unwrap())
        .collect::<Vec<u32>>();
    server_ee_params.not_after = rcgen::date_time_ymd(
        not_after_pieces[0] as i32,
        not_after_pieces[1] as u8,
        not_after_pieces[2] as u8,
    );

    let ee_key = KeyPair::generate_for(&PKCS_ED25519).unwrap();
    let server_cert = server_ee_params
        .signed_by(&ee_key, &ca_cert, &ca_key)
        .unwrap();

    let server_cert = rcgen::CertifiedKey {
        cert: server_cert,
        key_pair: ee_key,
    };

    // Generate the certificate
    info!("Writing certificate and key files");

    let cert_pem = server_cert.cert.pem();
    let key_pem = server_cert.key_pair.serialize_pem();

    std::fs::create_dir_all(&args.output_folder)
        .map_err(|e| e.to_string())?;
    std::fs::File::create(args.output_folder.join("cert.pem"))
        .map_err(|e| e.to_string())?
        .write_all(cert_pem.as_bytes())
        .map_err(|e| e.to_string())?;
    std::fs::File::create(args.output_folder.join("key.pem"))
        .map_err(|e| e.to_string())?
        .write_all(key_pem.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}
