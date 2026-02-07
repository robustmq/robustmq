// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use quinn::{ClientConfig, Endpoint};
use rustls::pki_types::CertificateDer;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

mod quic_connect_test;
mod quic_packet_test;

fn get_project_root() -> Result<String, Box<dyn std::error::Error>> {
    let tests_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let project_root = Path::new(&tests_manifest_dir)
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_string_lossy()
        .to_string();
    Ok(project_root)
}
fn open_ca_cert_file(file_path: &str) -> Result<fs::File, Box<dyn std::error::Error>> {
    let project_root = get_project_root().expect("Failed to get project root");
    let cert_path = Path::new(&project_root).join(file_path);
    Ok(fs::File::open(&cert_path)?)
}

fn parse_cert_file_to_buff_reader(
    cert_file: File,
) -> Result<BufReader<File>, Box<dyn std::error::Error>> {
    let cert_reader = BufReader::new(cert_file);
    Ok(cert_reader)
}

fn decode_cert_reader_to_certificate_der(
    mut cert_reader: &mut BufReader<File>,
) -> Vec<CertificateDer<'_>> {
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    certs
}

pub fn build_client_endpoint(addr: &str) -> quinn::Endpoint {
    // install the default TLS client config
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut roots = rustls::RootCertStore::empty();
    let cert_file = open_ca_cert_file("config/certs/ca.pem").expect("Failed to open CA cert file");
    let mut cert_reader =
        parse_cert_file_to_buff_reader(cert_file).expect("Failed to parse cert file");
    let certs = decode_cert_reader_to_certificate_der(&mut cert_reader);

    for cert in certs {
        roots.add(cert).unwrap();
    }

    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let quic_client_config =
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto).unwrap();

    let client_config = ClientConfig::new(Arc::new(quic_client_config));

    let mut endpoint = Endpoint::client(addr.parse().unwrap()).unwrap();
    endpoint.set_default_client_config(client_config);

    endpoint
}
