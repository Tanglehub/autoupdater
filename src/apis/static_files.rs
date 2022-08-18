use reqwest::{header, Url};
use crate::apis::{DownloadApiTrait, SimpleTag};
use crate::{Error, ReleaseAsset};
use std::{cmp::Ordering, fmt::Display};
use reqwest::header::HeaderMap;
use serde::Deserialize;

pub struct StaticFilesApi {
    url: Url,
    asset_name: Option<String>,
    current_version: Option<String>
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Clone)]
pub struct StaticFileAsset {
    pub name: String,
    pub url: String
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Clone)]
pub struct StaticFileRelease {
    pub version: String,
    pub assets: Vec<StaticFileAsset>,
}

impl StaticFilesApi {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            asset_name: None,
            current_version: None
        }
    }

    /// Sets current version of the application, this is used to determine if the latest release is newer than the current version.
    pub fn current_version(&mut self, current_version: &str) -> &mut Self {
        self.current_version = Some(current_version.to_string());
        self
    }

    /// Sets asset name to download.
    pub fn asset_name(&mut self, asset_name: &str) -> &mut Self {
        self.asset_name = Some(asset_name.to_string());
        self
    }

    fn get_releases(&self) -> Result<Vec<StaticFileRelease>, Error> {
        let api_url = format!(
            "{}/releases.json",
            self.url
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/updater".parse().expect("Invalid user agent"),
        );

        let response = reqwest::blocking::Client::new()
            .get(&api_url)
            .headers(headers)
            .send()?;

        let release_list: Vec<StaticFileRelease> = response.json()?;
        Ok(release_list)
    }

    fn match_releases<'releases>(
        &self,
        releases: &'releases [StaticFileRelease],
    ) -> Vec<&'releases StaticFileRelease> {
        releases
            .iter()
            .filter(|e| {
                let asset_name = match self.asset_name {
                    Some(ref asset_name) => e.assets.iter().any(|e| e.name == *asset_name),
                    None => true,
                };

                asset_name
            })
            .collect()
    }

    /// Gets the latest release
    pub fn send<Sort: Fn(&str, &str) -> Ordering>(
        &self,
        sort_func: &Option<Sort>,
    ) -> Result<StaticFileRelease, Error> {
        let mut releases = self.get_releases()?;

        let mut matching = self.match_releases(&releases);
        if matching.is_empty() {
            return Err(Error::no_release());
        }

        match sort_func {
            Some(sort_func) => {
                matching.sort_by(|a, b| sort_func(&a.version, &b.version));
            }
            None => matching.sort_by(|a, b| SimpleTag::simple_compare(&a.version, &b.version)),
        };

        let latest_release = matching.last().ok_or_else(Error::no_release)?;
        Ok((*latest_release).clone())
    }

    /// Gets the newest release if the newest release is newer than the current one.
    ///
    /// sort_func is used to compare two release versions if the format is not x.y.z
    pub fn get_newer(
        &self,
        sort_func: &Option<Box<dyn Fn(&str, &str) -> Ordering>>,
    ) -> Result<Option<StaticFileRelease>, Error> {
        let latest_release = self.send(sort_func)?;
        let is_newer = match self.current_version {
            Some(ref current_version) => {
                let newer = match sort_func {
                    Some(sort_func) => sort_func(&latest_release.version, current_version),
                    None => SimpleTag::simple_compare(&latest_release.version, current_version),
                };

                newer == Ordering::Greater
            }
            None => true,
        };

        if is_newer {
            Ok(Some(latest_release))
        } else {
            Ok(None)
        }
    }
}

impl DownloadApiTrait for StaticFilesApi {
    fn download<Asset: ReleaseAsset>(&self, asset: &Asset, download_callback: Option<Box<dyn Fn(f32)>>) -> Result<(), Error> {
        todo!()
    }
}