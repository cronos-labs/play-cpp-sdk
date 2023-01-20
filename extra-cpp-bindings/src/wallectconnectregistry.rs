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
    pub fn get_listing(&self, id: String) -> Result<&Listing, GameSdkError> {
        match self.listings.iter().find(|x| x.1.id == id) {
            Some((_, listing)) => Ok(listing),
            None => Err(GameSdkError::InvalidWalletId),
        }
    }

    pub(crate) fn check_wallet(
        &self,
        id: String,
        platform: Platform,
    ) -> Result<bool, GameSdkError> {
        let listing = self.get_listing(id)?;
        Ok(listing.supports_platform(&platform))
    }

    pub(crate) fn get_wallet(&self, id: String) -> Result<WalletEntry, GameSdkError> {
        match self.listings.iter().find(|x| x.1.id == id) {
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
            Platform::Mobile => {
                !self.mobile.native.clone().unwrap_or_default().is_empty()
                    || !self.mobile.universal.clone().unwrap_or_default().is_empty()
            }
            Platform::Desktop => {
                !self.desktop.native.clone().unwrap_or_default().is_empty()
                    || !self
                        .desktop
                        .universal
                        .clone()
                        .unwrap_or_default()
                        .is_empty()
            }
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
        assert_eq!(wallets.len(), 164);
        assert!(wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Desktop));
        assert_eq!(wallets.len(), 20);
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

    #[test]
    fn test_check_wallet() {
        let reg: Registry = Registry::default();

        // both desktop native and desktop universal are empty string
        let listing = reg
            .get_listing(
                "ffa139f74d1c8ebbb748cf0166f92d886e8c81b521c2193aa940e00626f4e215".to_string(),
            )
            .unwrap();
        assert_eq!(listing.desktop.native, Some("".to_string()));
        assert_eq!(listing.desktop.universal, Some("".to_string()));
        let valid = reg
            .check_wallet(
                "ffa139f74d1c8ebbb748cf0166f92d886e8c81b521c2193aa940e00626f4e215".to_string(),
                Platform::Desktop,
            )
            .unwrap();
        assert_eq!(valid, false);

        // both desktop native and desktop universal are null
        let listing = reg
            .get_listing(
                "0b415a746fb9ee99cce155c2ceca0c6f6061b1dbca2d722b3ba16381d0562150".to_string(),
            )
            .unwrap();
        assert_eq!(listing.desktop.native, None);
        assert_eq!(listing.desktop.universal, None);
        let valid = reg
            .check_wallet(
                "0b415a746fb9ee99cce155c2ceca0c6f6061b1dbca2d722b3ba16381d0562150".to_string(),
                Platform::Desktop,
            )
            .unwrap();
        assert_eq!(valid, false);

        // desktop native is null but desktop universal is empty string
        let listing = reg
            .get_listing(
                "107bb20463699c4e614d3a2fb7b961e66f48774cb8f6d6c1aee789853280972c".to_string(),
            )
            .unwrap();
        assert_eq!(listing.desktop.native, None);
        assert_eq!(listing.desktop.universal, Some("".to_string()));
        let valid = reg
            .check_wallet(
                "107bb20463699c4e614d3a2fb7b961e66f48774cb8f6d6c1aee789853280972c".to_string(),
                Platform::Desktop,
            )
            .unwrap();
        assert_eq!(valid, false);

        // mobile native is empty but mobile universal is null
        let listing = reg
            .get_listing(
                "23db748bbf7ba1e737921bee04f54d53356e95533e0ed66c39113324873294e7".to_string(),
            )
            .unwrap();
        assert_eq!(listing.mobile.native, Some("".to_string()));
        assert_eq!(listing.mobile.universal, None);
        let valid = reg
            .check_wallet(
                "23db748bbf7ba1e737921bee04f54d53356e95533e0ed66c39113324873294e7".to_string(),
                Platform::Mobile,
            )
            .unwrap();
        assert_eq!(valid, false);
    }
}
