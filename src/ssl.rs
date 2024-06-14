use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use std::fs::{self, File};
use std::path::PathBuf;
use dirs::data_local_dir;
use rcgen::{generate_simple_self_signed, Certificate, CertificateParams, KeyPair, date_time_ymd, DistinguishedName, DnType, SanType};
use log::info;
use openssl::x509::X509;

/// Load or create SSL certificates.
pub fn load_or_create_certificates() -> SslAcceptorBuilder {
    let ssl_dir = data_local_dir().unwrap().join(".wex/ssl");
    fs::create_dir_all(&ssl_dir).expect("Failed to create SSL directory");

    let cert_path = ssl_dir.join("cert.pem");
    let key_path = ssl_dir.join("key.pem");

    if !cert_path.exists() || !key_path.exists() || are_certificates_expired(&cert_path) {
        generate_certificates(&cert_path, &key_path);
    }

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(key_path, SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_file(cert_path, SslFiletype::PEM)
        .unwrap();

    builder
}

/// Check if the certificates are expired.
fn are_certificates_expired(cert_path: &PathBuf) -> bool {
    let cert_contents = fs::read(cert_path).expect("Failed to read certificate");
    let cert = X509::from_pem(&cert_contents).unwrap();
    cert.not_after() < &openssl::asn1::Asn1Time::days_from_now(0).unwrap()
}

/// Generate new certificates, and write to the specified paths, replacing existing files.
fn generate_certificates(cert_path: &PathBuf, key_path: &PathBuf) {
    let mut params = CertificateParams::default();
    params.not_before = date_time_ymd(1975, 1, 1);
    params.not_after = date_time_ymd(4096, 1, 1);
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "Crab Widgets SE");
    dn.push(DnType::CommonName, "Master Cert");
    params.distinguished_name = dn;
    params.subject_alt_names = vec![
        SanType::DnsName("crabs.crabs".try_into().unwrap()),
        SanType::DnsName("localhost".try_into().unwrap()),
    ];

    let key_pair = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key_pair).unwrap();

    let pem_serialized = cert.pem();
	println!("{pem_serialized}");
	println!("{}", key_pair.serialize_pem());
	fs::create_dir_all("certs/").unwrap();
	fs::write(cert_path, pem_serialized.as_bytes()).unwrap();
	fs::write(key_path, key_pair.serialize_pem().as_bytes()).unwrap();
}