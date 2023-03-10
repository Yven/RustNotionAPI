use super::{Notion, get_property_value, get_value_str, property::Property, property::Author, block::Block, Json, CommErr, NewImp};
use anyhow::Result;


// 页结构
#[derive(Debug)]
pub struct Page {
    pub id: String,
    pub created_time: String,
    pub edited_time: String,
    pub author: Author,
    pub editor_id: String,
    pub cover: String,
    pub icon: String,
    pub title: String,
    pub archived: bool,
    pub url: String,
    pub properties: Vec<Property>,
    pub content: Block,
}

impl NewImp for Page {
    fn new(page: &Json) -> Result<Self> {
        let property_list = page.get("properties").ok_or(CommErr::FormatErr("properties"))?;

        let author = Author::new(property_list)?;

        let mut properties: Vec<Property> = Vec::new();
        for (key, value) in property_list.as_object().ok_or(CommErr::FormatErr("properties"))?.iter() {
            match key.as_str() {
                "Author" | "Created time" | "Edited time" | "Name" => (),
                _ => properties.push(Property::new(key, value)?),
            }
        }

        Ok(Page {
            id: get_value_str(page, "id")?,
            created_time: get_value_str(page, "created_time")?,
            edited_time: get_value_str(page, "last_edited_time")?,
            author,
            editor_id: get_value_str(&page["last_edited_by"], "id").unwrap_or_default(),
            cover: get_value_str(page, "cover").unwrap_or_default(),
            icon: get_value_str(page, "icon").unwrap_or_default(),
            title: get_value_str(
                get_property_value(property_list, Some("Name"))?
                .get(0).ok_or(CommErr::FormatErr("properties"))?
            , "plain_text")?,
            archived: page.get("archived")
                .ok_or(CommErr::FormatErr("archived"))?
                .as_bool().unwrap_or_default(),
            url: get_value_str(page, "url").unwrap_or_default(),
            properties,
            content: Block::default(),
        })
    }
}

impl Page {
    pub fn content(&mut self) -> Result<String> {
        let block = Notion::Blocks(self.id.to_string()).search::<Block>()?;
        self.content = block;

        Ok(self.content.to_string())
    }

    pub fn search_property(&self, key: &str) -> Result<Vec<(String, String)>> {
        let mut res = Vec::new();
        for p in self.properties.iter() {
            if p.property.get_val() == key {
                use super::property::PropertyType::*;
                let data_key = match p.property {
                    MultiSelect(_) => "name",
                    Date(_) => "date",
                    Text(_) => "plain_text",
                    _ => "name",
                };
                let msg: &'static str = Box::leak(Box::new("MultiSelect".to_string() + key));
                for p_item in p.data.iter() {
                    res.push((
                        p_item.get(data_key).ok_or(CommErr::FormatErr(msg))?.to_string(),
                        p_item.get("id").unwrap_or(&String::default()).to_string(),
                    ));
                }
                break;
            }
        }

        Ok(res)
    }
}
