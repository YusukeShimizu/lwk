use std::collections::HashMap;

use common::Signer;
use signer::AnySigner;
use tiny_jrpc::error::Error as TinyRpcError;
use wollet::bitcoin::bip32::Fingerprint;
use wollet::Wollet;

use crate::config::Config;

pub enum AppSigner {
    AvailableSigner(AnySigner),
    ExternalSigner(Fingerprint),
}

impl AppSigner {
    pub fn fingerprint(&self) -> Fingerprint {
        match self {
            AppSigner::AvailableSigner(s) => s.fingerprint().unwrap(), // TODO
            AppSigner::ExternalSigner(f) => *f,
        }
    }
}

#[derive(Default)]
pub struct Wollets(HashMap<String, Wollet>);

#[derive(Default)]
pub struct Signers(HashMap<String, AppSigner>);

#[derive(Default)]
pub struct State {
    // TODO: config is read-only, so it's not useful to wrap it in a mutex.
    // Ideally it should be in _another_ struct accessible by method_handler.
    pub config: Config,
    pub wollets: Wollets,
    pub signers: Signers,
}

impl Wollets {
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> tiny_jrpc::Result<&Wollet> {
        self.0
            .get(name)
            .ok_or_else(|| TinyRpcError::WalletNotExist(name.to_string()))
    }

    pub fn get_mut(&mut self, name: &str) -> tiny_jrpc::Result<&mut Wollet> {
        self.0
            .get_mut(name)
            .ok_or_else(|| TinyRpcError::WalletNotExist(name.to_string()))
    }

    pub fn insert(&mut self, name: &str, wollet: Wollet) -> tiny_jrpc::Result<()> {
        if self.0.contains_key(name) {
            return Err(TinyRpcError::WalletAlreadyLoaded(name.to_string()));
        }

        let a = |w: &Wollet| w.address(Some(0)).unwrap().address().to_string();

        let vec: Vec<_> = self
            .0
            .iter()
            .filter(|(_, w)| a(w) == a(&wollet))
            .map(|(n, _)| n)
            .collect();
        if let Some(existing) = vec.first() {
            // TODO: maybe a different error more clear?
            return Err(TinyRpcError::WalletAlreadyLoaded(existing.to_string()));
        }

        self.0.insert(name.to_string(), wollet);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> tiny_jrpc::Result<Wollet> {
        self.0
            .remove(name)
            .ok_or_else(|| TinyRpcError::WalletNotExist(name.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Wollet)> {
        self.0.iter()
    }
}

impl Signers {
    pub fn get(&self, name: &str) -> tiny_jrpc::Result<&AppSigner> {
        self.0
            .get(name)
            .ok_or_else(|| TinyRpcError::SignerNotExist(name.to_string()))
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, name: &str) -> tiny_jrpc::Result<&mut AppSigner> {
        self.0
            .get_mut(name)
            .ok_or_else(|| TinyRpcError::SignerNotExist(name.to_string()))
    }

    pub fn get_available(&self, name: &str) -> tiny_jrpc::Result<&AnySigner> {
        match self.get(name)? {
            AppSigner::AvailableSigner(signer) => Ok(signer),
            AppSigner::ExternalSigner(_) => Err(TinyRpcError::Generic(
                "Invalid operation for external signer".to_string(),
            )),
        }
    }

    pub fn insert(&mut self, name: &str, signer: AppSigner) -> tiny_jrpc::Result<()> {
        if self.0.contains_key(name) {
            return Err(TinyRpcError::SignerAlreadyLoaded(name.to_string()));
        }

        // TODO: matchin for fingerprint is not ideal, we could have collisions
        let vec: Vec<_> = self
            .0
            .iter()
            .filter(|(_, s)| s.fingerprint() == signer.fingerprint())
            .map(|(n, _)| n)
            .collect();
        if let Some(existing) = vec.first() {
            // TODO: maybe a different error more clear?
            return Err(TinyRpcError::SignerAlreadyLoaded(existing.to_string()));
        }

        self.0.insert(name.to_string(), signer);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> tiny_jrpc::Result<AppSigner> {
        self.0
            .remove(name)
            .ok_or_else(|| TinyRpcError::SignerNotExist(name.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &AppSigner)> {
        self.0.iter()
    }
}
