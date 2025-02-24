use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use log::{error, info};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    ServerConfig,
};
use rustls_pemfile::{certs, rsa_private_keys};

pub fn load_tls_config() -> Option<Arc<ServerConfig>> {
    let cert_path_str = "cert.pem";
    let key_path_str = "key.pem";

    let cert_path = Path::new(cert_path_str);
    let key_path = Path::new(key_path_str);

    if !cert_path.exists() || !key_path.exists() {
        info!("TLS 設定なし：平文のSMTPで動作します");
        return None;
    }

    let certs = CertificateDer::pem_file_iter(cert_path)
        .map_err(|e| error!("{}", e))
        .ok()?
        .map(|cert| cert.unwrap())
        .collect();

    let key = PrivateKeyDer::from_pem_file(key_path)
        .map_err(|e| error!("{}", e))
        .ok()?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| error!("{}", e))
        .ok()?;

    info!("TLSの設定完了");
    Some(Arc::new(config))
}
