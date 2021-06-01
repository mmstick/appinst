use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dep11Package {
    #[serde(rename = "Type")]
    pub type_: String,

    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "Name")]
    pub name: HashMap<String, String>,

    #[serde(rename = "Icon")]
    pub icon: Option<Icon>,

    #[serde(rename = "Package")]
    pub package: String,

    #[serde(rename = "Summary")]
    pub summary: HashMap<String, String>,

    #[serde(rename = "Description")]
    pub description: Option<HashMap<String, String>>,

    #[serde(rename = "Categories")]
    pub categories: Option<Vec<String>>,

    // #[serde(rename = "Keywords")]
    // pub keywords: Option<Vec<String>>,

    #[serde(rename = "ProjectLicense")]
    pub license: Option<String>,

    #[serde(rename = "Url")]
    pub urls: Option<HashMap<String, String>>,

    #[serde(rename = "Launchable")]
    pub launchable: Option<Launchable>,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Icon {
    pub cached: Option<Vec<CachedIcon>>,
    pub stock: Option<String>,
    pub remote: Option<Vec<RemoteIcon>>,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CachedIcon {
    pub name: String,
    pub width: u16,
    pub height: u16,
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RemoteIcon {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Launchable {
    #[serde(rename = "desktop-id")]
    pub desktop_id: Vec<String>
}