#[derive(Debug, Clone)]
pub(crate) enum TagAttribute {
    Names(Vec<String>),
    Path(String),
    About(String),
    App(String),
}

pub(crate) fn tag_attribute_parser(s: &str) -> Result<TagAttribute, &'static str> {
    let Some((attrib, val)) = s.split_once('=') else {
        return Err("expected valid tag attribute and value");
    };

    Ok(match attrib {
        "name" | "N" => TagAttribute::Names(val.split(',').map(String::from).collect()),
        "path" | "P" => TagAttribute::Path(val.to_string()),
        "about" => TagAttribute::About(val.to_string()),
        "default-app" | "D" => TagAttribute::App(val.to_string()),
        _ => return Err("invalid tag attribute"),
    })
}
