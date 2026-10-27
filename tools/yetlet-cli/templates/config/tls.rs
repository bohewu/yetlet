// Application-owned Development TLS; dependencies are absent unless selected.
#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tls {
    pub enabled: bool,
    pub certificate: String,
    pub private_key: String,
}

#[cfg(feature = "yetlet-dev-tls")]
pub type Transport = Option<axum_server::tls_rustls::RustlsConfig>;
#[cfg(not(feature = "yetlet-dev-tls"))]
pub type Transport = ();

impl Settings {
    pub async fn tls(&self) -> io::Result<Transport> {
        let _ = (&self.server.tls.certificate, &self.server.tls.private_key);
        if !self.server.tls.enabled {
            #[cfg(feature = "yetlet-dev-tls")]
            return Ok(None);
            #[cfg(not(feature = "yetlet-dev-tls"))]
            return Ok(());
        }
        if std::env::var_os("APP_ENVIRONMENT").as_deref()
            != Some(std::ffi::OsStr::new("Development"))
        {
            return Err(invalid());
        }
        #[cfg(not(feature = "yetlet-dev-tls"))]
        return Err(io::Error::other("Development TLS requires the yetlet-dev-tls Cargo feature"));
        #[cfg(feature = "yetlet-dev-tls")]
        {
            let certificate = self.tls_path(&self.server.tls.certificate)?;
            let key = self.tls_path(&self.server.tls.private_key)?;
            if certificate == key {
                return Err(invalid());
            }
            let _ = rustls::crypto::ring::default_provider().install_default();
            axum_server::tls_rustls::RustlsConfig::from_pem_file(certificate, key)
                .await
                .map(Some)
                .map_err(|_| io::Error::other("Development TLS certificate or private key invalid"))
        }
    }

    #[cfg(feature = "yetlet-dev-tls")]
    fn tls_path(&self, value: &str) -> io::Result<PathBuf> {
        use std::path::{Component, Path};
        let relative = Path::new(value);
        if value.is_empty()
            || value.contains([':', '\\', '\0', '\n', '\r'])
            || value.split('/').any(|p| matches!(p, "" | "." | ".."))
            || relative.components().any(|p| !matches!(p, Component::Normal(_)))
            || relative.starts_with(@FRONTEND_ROOT@)
        {
            return Err(invalid());
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        for component in relative.components() {
            path.push(component);
            let metadata = std::fs::symlink_metadata(&path).map_err(|_| invalid())?;
            if metadata.file_type().is_symlink() {
                return Err(invalid());
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err(invalid());
                }
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o022 != 0 {
                    return Err(invalid());
                }
            }
        }
        if !path.is_file()
            || !path.canonicalize().map_err(|_| invalid())?
                .starts_with(root.canonicalize().map_err(|_| invalid())?)
        {
            return Err(invalid());
        }
        Ok(path)
    }
}

pub async fn serve(
    listener: tokio::net::TcpListener,
    tls: Transport,
    app: axum::Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    #[cfg(feature = "yetlet-dev-tls")]
    if let Some(tls) = tls {
        let handle = axum_server::Handle::new();
        let server = axum_server::from_tcp_rustls(listener.into_std()?, tls)?
            .handle(handle.clone())
            .serve(app.into_make_service());
        tokio::pin!(server);
        tokio::select! {
            result = &mut server => return result,
            _ = shutdown => {
                handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
                return server.await;
            }
        }
    }
    #[cfg(not(feature = "yetlet-dev-tls"))]
    let _ = tls;
    axum::serve(listener, app).with_graceful_shutdown(shutdown).await
}

#[cfg(test)]
mod development_tls_tests {
    use super::*;

    #[tokio::test]
    async fn disabled_tls_does_not_open_development_certificate_files() {
        let mut settings: Settings = loader()
            .load_with_env([("APP_SERVICE__TOKEN".into(), "synthetic-token".into())])
            .unwrap();
        settings.server.tls.certificate = ".dev/missing-certificate.pem".into();
        settings.server.tls.private_key = ".dev/missing-private-key.pem".into();
        assert!(settings.tls().await.is_ok());
    }

    #[cfg(feature = "yetlet-dev-tls")]
    #[test]
    fn tls_paths_reject_traversal_frontend_and_missing_files_with_safe_errors() {
        let settings: Settings = loader().load_with_env([]).unwrap();
        for path in ["../private.pem", "/private.pem", "frontend/private.pem", ".dev/missing-private.pem"] {
            let error = settings.tls_path(path).unwrap_err();
            assert_eq!(error.to_string(), "application configuration invalid");
            assert!(!format!("{error:?}").contains(path));
        }
    }
}
