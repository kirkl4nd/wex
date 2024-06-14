use dirs::data_local_dir;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use openssl::x509::X509;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::fs::{self};
use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;

/// Load or create SSL certificates.
pub fn load_or_create_certificates() -> io::Result<SslAcceptorBuilder> {
    let ssl_dir = data_local_dir()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Local data directory not found"))?
        .join(".wex/ssl");
    fs::create_dir_all(&ssl_dir).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to create SSL directory: {}", e),
        )
    })?;

    let cert_path = ssl_dir.join("cert.pem");
    let key_path = ssl_dir.join("key.pem");

    if !cert_path.exists() || !key_path.exists() || are_certificates_expired(&cert_path)? {
        generate_certificates(&cert_path, &key_path)?;
    }

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to create SSL acceptor"))?;
    builder
        .set_private_key_file(&key_path, SslFiletype::PEM)
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to set private key"))?;
    builder
        .set_certificate_file(&cert_path, SslFiletype::PEM)
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to set certificate"))?;

    Ok(builder)
}

/// Check if the certificates are expired.
fn are_certificates_expired(cert_path: &PathBuf) -> io::Result<bool> {
    let cert_contents = fs::read(cert_path).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to read certificate: {}", e),
        )
    })?;
    let cert = X509::from_pem(&cert_contents)
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to parse certificate"))?;
    Ok(cert.not_after() < &openssl::asn1::Asn1Time::days_from_now(0).unwrap())
}

/// Generate new certificates, and write to the specified paths, replacing existing files.
fn generate_certificates(cert_path: &PathBuf, key_path: &PathBuf) -> io::Result<()> {
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

    let key_pair = KeyPair::generate()
        .map_err(|_| Error::new(ErrorKind::Other, "Failed to generate key pair"))?;
    let cert = params.self_signed(&key_pair).map_err(|_| {
        Error::new(
            ErrorKind::Other,
            "Failed to generate self-signed certificate",
        )
    })?;

    let pem_serialized = cert.pem();

    fs::write(cert_path, pem_serialized.as_bytes()).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to write certificate: {}", e),
        )
    })?;
    fs::write(key_path, key_pair.serialize_pem().as_bytes())
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to write key: {}", e)))?;

    Ok(())
}
