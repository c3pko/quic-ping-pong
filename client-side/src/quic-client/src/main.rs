use std::iter;
use std::{error::Error, fs::File, io::BufReader};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use quinn::{ClientConfig, Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use pem::parse;
use rcgen::Certificate;
use rustls_pemfile::{Item, read_one, certs};
use x509_parser::pem::pem_to_der;
use x509_parser::parse_x509_der;

#[allow(unused)]
pub fn make_client_endpoint(
    bind_addr: SocketAddr,
    server_certs: &[&[u8]],
) -> Result<Endpoint, Box<dyn Error>> {
    let client_cfg = configure_client(server_certs)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);
    let client_config = ClientConfig::with_native_roots();

    Ok(endpoint)
}

/// Attempt QUIC connection with the given server address.
async fn run_client(endpoint: &Endpoint, server_addr: SocketAddr) {
   let connect = endpoint.connect(server_addr, "localhost").unwrap();
   let connection = connect.await.unwrap();
   println!("[client] connected: addr={}", connection.remote_address());
}


fn configure_client(server_certs: &[&[u8]]) -> Result<ClientConfig, Box<dyn Error>> {
    /*let mut certs = rustls::RootCertStore::empty();
    for cert in server_certs {
        certs.add(&rustls::Certificate(cert.to_vec()))?;
    }

    Ok(ClientConfig::with_root_certificates(certs))*/
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    Ok(ClientConfig::new(Arc::new(crypto)))
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        println!("verify: {:?}", _server_name);
        println!("verify: {:?}", _intermediates);
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

/// Runs a QUIC server bound to given address and returns server certificate.
fn run_server(addr: SocketAddr) -> Result<Vec<u8>, Box<dyn Error>> {
    let (endpoint, server_cert) = make_server_endpoint(addr)?;
    // accept a single connection
    tokio::spawn(async move {
        let connection = endpoint.accept().await.unwrap().await.unwrap();
        println!(
            "[server] incoming connection: addr={}",
            connection.remote_address()
        );
    });

    Ok(server_cert)
}

#[allow(unused)]
pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, Vec<u8>), Box<dyn Error>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

#[allow(clippy::field_reassign_with_default)] // https://github.com/rust-lang/rust-clippy/issues/6527
fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    use std::io::Write;
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let cert_pem = cert.serialize_pem().unwrap();

    let mut file = File::create("server.crt")?;
    file.write_all(cert_pem.as_bytes());
    drop(file);

    let priv_key = cert.serialize_private_key_der();
    let priv_key_pem = cert.serialize_private_key_pem();

    let mut file = File::create("server.key")?;
    file.write_all(priv_key_pem.as_bytes());
    drop(file);

    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0_u8.into());

    Ok((server_config, cert_der))
}

fn print_type_of<T>(_: &T) {
    
    println!("{}", std::any::type_name::<T>());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verify_cert_format() {
        let cert_address = "127.0.0.1:5003".parse().unwrap();
        let server_cert = run_server(cert_address);
        println!("server_cert={:?}", server_cert);
        print_type_of(&server_cert);
        //verify cert is a vector

    }


}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let cert_address = "127.0.0.1:5001".parse().unwrap();
    let server_address = "127.0.0.1:5000".parse().unwrap();
    //calls server to get credentials, creates client, tries to connect to Python server (at 127.00.0.1:5000)
    let server_cert = run_server(cert_address)?;

    let client = make_client_endpoint(
        "127.0.0.1:0".parse().unwrap(),
        &[&server_cert],
    )?;

    tokio::join!(
        run_client(&client, server_address),
    );

   client.wait_idle().await;
    
   println!("client connected");

   Ok(())
}