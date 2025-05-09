#[derive(Debug, Clone)]
pub(crate) enum TagAttribute {
    Names(Vec<String>),
    Path(String),
    About(String),
    App(String),
}

pub(crate) fn update_tag_parser(s: &str) -> Result<TagAttribute, String> {
    let Some((attrib, val)) = s.split_once('=') else {
        return Err("expected valid tag attribute and value".to_string());
    };

    Ok(match attrib {
        "name" | "N" => TagAttribute::Names(val.split(',').map(String::from).collect()),
        "path" | "P" => TagAttribute::Path(val.to_string()),
        "about" => TagAttribute::About(val.to_string()),
        "default-app" | "D" => TagAttribute::App(val.to_string()),
        _ => return Err("invalid tag attribute".to_string()),
    })
}
