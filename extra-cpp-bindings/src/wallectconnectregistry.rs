use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::GameSdkError;
use crate::{ImageUrl, Platform, WalletEntry};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Registry {
    listings: BTreeMap<String, Listing>,
    count: i64,
    total: i64,
}

impl Registry {
    pub(crate) fn fetch_new(cache: Option<PathBuf>) -> Result<Self, GameSdkError> {
        const URL: &str = "https://registry.walletconnect.com/api/v2/wallets";
        let client = reqwest::blocking::Client::new();
        let resp: Registry = client.get(URL).send()?.json()?;
        if let Some(cache) = cache {
            std::fs::write(cache, serde_json::to_string(&resp)?)?;
        }
        Ok(resp)
    }
    pub(crate) fn load_cached(cache: Option<PathBuf>) -> Result<Self, GameSdkError> {
        if let Some(cache) = cache {
            let data = std::fs::read_to_string(cache)?;
            let registry: Registry = serde_json::from_str(&data)?;
            return Ok(registry);
        }
        Ok(Self::default())
    }
}

impl Default for Registry {
    fn default() -> Self {
        // get default cached value
        serde_json::from_str(include_str!("registry.json")).unwrap()
    }
}

impl Registry {
    pub(crate) fn get_wallet(&self, id: String) -> Result<WalletEntry, GameSdkError> {
        match self.listings.iter().filter(|x| x.1.id == id).next() {
            Some((_, listing)) => Ok(WalletEntry {
                id: listing.id.clone(),
                name: listing.name.clone(),
                image_url: listing.image_url.clone(),
                mobile_native_link: listing.mobile.native.clone().unwrap_or_default(),
                mobile_universal_link: listing.mobile.universal.clone().unwrap_or_default(),
                desktop_native_link: listing.desktop.native.clone().unwrap_or_default(),
                desktop_universal_link: listing.desktop.universal.clone().unwrap_or_default(),
            }),
            None => Err(GameSdkError::InvalidWalletId),
        }
    }

    pub(crate) fn filter_wallets(&self, platform: Option<Platform>) -> Vec<WalletEntry> {
        let mut filtered = Vec::new();
        for (_, listing) in self.listings.iter() {
            if let Some(ref platform) = platform {
                if !listing.supports_platform(platform) {
                    continue;
                }
            }
            filtered.push(WalletEntry {
                id: listing.id.clone(),
                name: listing.name.clone(),
                image_url: listing.image_url.clone(),
                mobile_native_link: listing.mobile.native.clone().unwrap_or_default(),
                mobile_universal_link: listing.mobile.universal.clone().unwrap_or_default(),
                desktop_native_link: listing.desktop.native.clone().unwrap_or_default(),
                desktop_universal_link: listing.desktop.universal.clone().unwrap_or_default(),
            });
        }
        filtered
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Listing {
    id: String,
    name: String,
    description: Option<String>,
    homepage: String,
    chains: Vec<String>,
    versions: Vec<String>,
    sdks: Vec<Sdk>,
    app_type: AppType,
    image_id: String,
    image_url: ImageUrl,
    app: App,
    mobile: Desktop,
    desktop: Desktop,
    metadata: Metadata,
}

impl Listing {
    fn supports_platform(&self, platform: &Platform) -> bool {
        match *platform {
            Platform::Mobile => self.mobile.native.is_some() || self.mobile.universal.is_some(),
            Platform::Desktop => self.desktop.native.is_some() || self.desktop.universal.is_some(),
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct App {
    browser: Option<String>,
    ios: Option<String>,
    android: Option<String>,
    mac: Option<String>,
    windows: Option<String>,
    linux: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Desktop {
    native: Option<String>,
    universal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Metadata {
    #[serde(rename = "shortName")]
    short_name: Option<String>,
    colors: Colors,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Colors {
    primary: Option<String>,
    secondary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum AppType {
    #[serde(rename = "hybrid")]
    Hybrid,
    #[serde(rename = "wallet")]
    Wallet,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Sdk {
    #[serde(rename = "sign_v1")]
    SignV1,
    #[serde(rename = "sign_v2")]
    SignV2,
    #[serde(rename = "auth_v1")]
    AuthV1,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn parse_registry() {
        let reg: Registry = Registry::default();
        const DEFI_WALLET_NAME: &str = "Crypto.com | DeFi Wallet";
        assert_eq!(reg.count, 246);
        let wallets = reg.filter_wallets(None);
        assert_eq!(wallets.len(), 246);
        assert!(wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Mobile));
        assert_eq!(wallets.len(), 212);
        assert!(wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Desktop));
        assert_eq!(wallets.len(), 120);
    }

    #[test]
    fn test_fetch_new_no_cache() {
        let reg = Registry::fetch_new(None);
        assert!(reg.is_ok());
    }

    #[test]
    fn test_fetch_new_with_cache() {
        let mut dir = std::env::temp_dir();
        let tmpfile = format!("{}.json", uuid::Uuid::new_v4());
        dir.push(tmpfile);
        println!("{:?}", dir);

        let reg = Registry::fetch_new(Some(dir.clone()));
        assert!(reg.is_ok());

        let reg = Registry::load_cached(Some(dir));
        assert!(reg.is_ok());
    }

    #[test]
    fn test_load_cached() {
        let reg = Registry::load_cached(Some(PathBuf::from("./not_exists.json")));
        assert!(reg.is_err());

        let reg = Registry::load_cached(None);
        assert!(reg.is_ok());
    }

    #[test]
    fn test_get_wallet() {
        let reg: Registry = Registry::default();
        let wallet = reg
            .get_wallet(
                "f2436c67184f158d1beda5df53298ee84abfc367581e4505134b5bcf5f46697d".to_string(),
            )
            .unwrap();
        assert_eq!(wallet.name, "Crypto.com | DeFi Wallet".to_string());
        assert_eq!(wallet.mobile_native_link, "cryptowallet:".to_string());
        assert_eq!(
            wallet.mobile_universal_link,
            "https://wallet.crypto.com".to_string()
        );
        assert_eq!(wallet.desktop_native_link, "cryptowallet:".to_string());
        assert_eq!(wallet.desktop_universal_link, "".to_string());

        let wallet = reg
            .get_wallet(
                "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96".to_string(),
            )
            .unwrap();
        assert_eq!(wallet.name, "MetaMask".to_string());
        assert_eq!(wallet.mobile_native_link, "metamask:".to_string());
        assert_eq!(
            wallet.mobile_universal_link,
            "https://metamask.app.link".to_string()
        );
        assert_eq!(wallet.desktop_native_link, "".to_string());
        assert_eq!(wallet.desktop_universal_link, "".to_string());

        let wallet = reg
            .get_wallet(
                "4622a2b2d6af1c9844944291e5e7351a6aa24cd7b23099efac1b2fd875da31a0".to_string(),
            )
            .unwrap();
        assert_eq!(wallet.name, "Trust Wallet".to_string());
        assert_eq!(wallet.mobile_native_link, "trust:".to_string());
        assert_eq!(
            wallet.mobile_universal_link,
            "https://link.trustwallet.com".to_string()
        );
        assert_eq!(wallet.desktop_native_link, "".to_string());
        assert_eq!(wallet.desktop_universal_link, "".to_string());
    }
}
