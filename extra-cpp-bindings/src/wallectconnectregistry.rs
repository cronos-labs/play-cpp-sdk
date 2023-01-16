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
    pub(crate) fn filter_wallets(&self, platform: Option<Platform>) -> Vec<WalletEntry> {
        let mut filtered = Vec::new();
        for (_, listing) in self.listings.iter() {
            if let Some(ref platform) = platform {
                if !listing.supports_platform(platform) {
                    continue;
                }
            }
            let native_link = if listing.mobile.native.is_some() {
                listing.mobile.native.clone()
            } else {
                listing.desktop.native.clone()
            };
            let universal_link = if listing.mobile.universal.is_some() {
                listing.mobile.universal.clone()
            } else {
                listing.desktop.universal.clone()
            };
            filtered.push(WalletEntry {
                name: listing.name.clone(),
                image_url: listing.image_url.clone(),
                native_link: native_link.unwrap_or_default(),
                universal_link: universal_link.unwrap_or_default(),
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
            Platform::Android => self.app.android.is_some(),
            Platform::Ios => self.app.ios.is_some(),
            Platform::Linux => self.app.linux.is_some(),
            Platform::Mac => self.app.mac.is_some(),
            Platform::Windows => self.app.windows.is_some(),
            Platform::Browser => self.app.browser.is_some(),
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
        let wallets = reg.filter_wallets(Some(Platform::Android));
        assert_eq!(wallets.len(), 217);
        assert!(wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Ios));
        assert_eq!(wallets.len(), 226);
        assert!(wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Linux));
        assert_eq!(wallets.len(), 93);
        assert!(!wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Windows));
        assert_eq!(wallets.len(), 99);
        assert!(!wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Mac));
        assert_eq!(wallets.len(), 101);
        assert!(!wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
        let wallets = reg.filter_wallets(Some(Platform::Browser));
        assert_eq!(wallets.len(), 182);
        assert!(!wallets.iter().any(|w| w.name == DEFI_WALLET_NAME));
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
}
