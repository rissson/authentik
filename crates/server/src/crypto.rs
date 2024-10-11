use anyhow::Result;
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod, SslVersion},
    x509::{
        X509, X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};

pub(crate) fn get_tls_acceptor_builder() -> Result<SslAcceptorBuilder> {
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server())?;
    builder.set_min_proto_version(Some(SslVersion::TLS1_2))?;
    Ok(builder)
}

pub(crate) fn generate_self_signed_cert() -> Result<(X509, PKey<Private>)> {
    let rsa = Rsa::generate(2048)?;
    let pkey = PKey::from_rsa(rsa)?;

    let mut x509_builder = X509::builder()?;

    x509_builder.set_version(2)?;

    let mut serial_number = BigNum::new()?;
    serial_number.rand(128, MsbOption::MAYBE_ZERO, true)?;
    x509_builder.set_serial_number(serial_number.to_asn1_integer()?.as_ref())?;

    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("CN", "authentik default certificate")?;
    name_builder.append_entry_by_text("O", "authentik")?;
    let name = name_builder.build();
    x509_builder.set_subject_name(&name)?;
    x509_builder.set_issuer_name(&name)?;

    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(365)?;
    x509_builder.set_not_before(&not_before)?;
    x509_builder.set_not_after(&not_after)?;

    let key_usage = KeyUsage::new().digital_signature().key_encipherment().build()?;
    x509_builder.append_extension(key_usage)?;

    let extended_key_usage = ExtendedKeyUsage::new().server_auth().build()?;
    x509_builder.append_extension(extended_key_usage)?;

    let subject_alt_name = SubjectAlternativeName::new()
        .dns("*")
        .build(&x509_builder.x509v3_context(None, None))?;
    x509_builder.append_extension(subject_alt_name)?;

    let basic_constraints = BasicConstraints::new().critical().ca().build()?;
    x509_builder.append_extension(basic_constraints)?;

    x509_builder.set_pubkey(&pkey)?;

    x509_builder.sign(&pkey, MessageDigest::sha256())?;

    let cert = x509_builder.build();

    Ok((cert, pkey))
}
