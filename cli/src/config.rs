#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::output::OutputFormat;

lazy_static! {
    pub static ref ESC_DIR: PathBuf = {
        let home_dir = dirs::home_dir().expect("Not supported platform: can't find home directory");
        Path::new(&home_dir).join(".esc")
    };
}

lazy_static! {
    pub static ref SETTINGS_FILE: PathBuf = Path::new(ESC_DIR.as_path()).join("settings.toml");
}

lazy_static! {
    pub static ref SETTINGS: Settings = {
        match load_settings() {
            Ok(settings) => settings,
            Err(e) => {
                if std::path::Path::new(SETTINGS_FILE.as_path()).exists() {
                    eprintln!(
                        "Error when parsing {}, fallback to default settings. Error: {}\n",
                        SETTINGS_FILE.as_path().display(),
                        e
                    );
                }

                Default::default()
            }
        }
    };
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub default_profile: Option<String>,

    #[serde(rename = "profile")]
    pub profiles: Vec<Profile>,
}

impl Settings {
    pub async fn configure() -> Result<(), Box<dyn std::error::Error>> {
        if tokio::fs::metadata(ESC_DIR.as_path()).await.is_err() {
            tokio::fs::create_dir_all(ESC_DIR.as_path()).await?;
            tokio::fs::File::create(SETTINGS_FILE.as_path()).await?;

            #[cfg(not(target_os = "windows"))]
            {
                let mut settings_file_permissions = tokio::fs::metadata(SETTINGS_FILE.as_path())
                    .await?
                    .permissions();

                settings_file_permissions.set_mode(0o640);
            }
        }

        Ok(())
    }

    pub async fn persist(self) -> Result<(), Box<dyn std::error::Error>> {
        if tokio::fs::metadata(ESC_DIR.as_path()).await.is_err() {
            tokio::fs::create_dir_all(ESC_DIR.as_path()).await?;
        }

        let bytes = toml::to_vec(&self)?;

        tokio::fs::write(SETTINGS_FILE.as_path(), &bytes).await?;

        Ok(())
    }

    pub fn get_current_profile(&self) -> Option<&Profile> {
        let default_profile_name = self.default_profile.as_ref()?.as_str();

        self.get_profile(default_profile_name)
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn get_profile_mut(&mut self, name: &str) -> &mut Profile {
        let mut idx = 0;
        let mut found = false;

        for profile in self.profiles.iter() {
            if profile.name == name {
                found = true;
                break;
            }

            idx += 1;
        }

        if found {
            return self
                .profiles
                .get_mut(idx)
                .expect("Impossible situation: we know idx is valid!");
        }

        self.profiles.push(Profile {
            name: name.to_string(),
            ..Default::default()
        });

        self.profiles
            .last_mut()
            .expect("Impossible situation: we just added a new profile!")
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TokenConfigOpts {
    pub audience: Option<String>,
    pub client_id: Option<String>,
    pub idp_client_id: Option<String>,
    pub identity_url: Option<String>,
    pub idp_kit_url: Option<String>,
    pub idp_um_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<esc_api::OrgId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<esc_api::ProjectId>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_url",
        serialize_with = "serialize_url",
        default
    )]
    pub api_base_url: Option<url::Url>,

    #[serde(rename = "fmt", skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OutputFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_config: Option<TokenConfigOpts>,
}

struct InvalidUrl {}

pub fn parse_url(str: &str) -> Result<url::Url, Box<dyn std::error::Error>> {
    let url = url::Url::parse(str)?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(InvalidUrl {}.into());
    }

    if url.host().is_none() {
        return Err(InvalidUrl {}.into());
    }

    Ok(url)
}

impl std::fmt::Display for InvalidUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Malformed URL. Expecting HTTP/HTTPS scheme, a valid host or IP"
        )
    }
}

impl std::fmt::Debug for InvalidUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Malformed URL. Expecting HTTP/HTTPS scheme, a valid host or IP"
        )
    }
}

impl std::error::Error for InvalidUrl {}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<url::Url>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(UrlVisitor {})
}

fn serialize_url<S>(url: &Option<url::Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(url) = url {
        return serializer.serialize_some(url.as_str());
    }

    serializer.serialize_none()
}

fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(SETTINGS_FILE.as_path())?;
    let settings: Settings = toml::from_slice(&bytes)?;

    Ok(settings)
}

struct UrlVisitor {}

impl<'a> serde::de::Visitor<'a> for UrlVisitor {
    type Value = Option<url::Url>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid URL")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match parse_url(value) {
            Ok(url) => Ok(Some(url)),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}
