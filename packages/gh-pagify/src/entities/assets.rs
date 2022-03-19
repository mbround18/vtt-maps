use std::collections::HashMap;

pub struct Asset {
    pub has_details: bool,
    pub has_preview: bool,
    pub has_vtt_file: bool,
    pub path: String,
    pub tags: Vec<String>,
}

impl Asset {
    pub fn new(path: String, tags: Vec<String>, boolean_list: &mut HashMap<String, bool>) -> Self {
        Asset {
            path,
            tags,
            has_preview: boolean_list.get("has_preview").unwrap_or(&false).to_owned(),
            has_details: boolean_list.get("has_details").unwrap_or(&false).to_owned(),
            has_vtt_file: boolean_list
                .get("has_vtt_file")
                .unwrap_or(&false)
                .to_owned(),
        }
    }
}
