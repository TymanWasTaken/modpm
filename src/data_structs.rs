use serde::Deserialize;

pub struct PolyInstance {
    pub name: String,
    pub folder_name: String,
    pub game_version: String,
    pub modloader: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct PolyInstanceDataComponent {
    pub uid: String,
    pub version: String,
}
#[derive(Deserialize)]
pub struct PolyInstanceDataJson {
    pub components: Vec<PolyInstanceDataComponent>,
}
