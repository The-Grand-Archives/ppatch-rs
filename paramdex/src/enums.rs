use serde_derive::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectEnums {
    pub list: Vec<ProjectEnum>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProjectEnum {
    pub display_name: String,
    pub name: String,
    pub description: String,
    pub options: Vec<EnumOption>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EnumOption {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub description: String,
}
