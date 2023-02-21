use super::{request::Request, request::RequestMethod, Module, get_property_value, get_value_str, property::Property, block::Block, Json, ImpRequest};
use anyhow::{Result, anyhow};


// 作者信息
#[allow(dead_code)]
#[derive(Debug)]
pub struct Author {
    id: String,
    name: String,
    avatar_url: String,
    email: String,
    user_type: String
}

impl Author {
    pub fn new(property_list: &Json) -> Result<Self> {
        let author = get_property_value(property_list, Some("Author"))?;
        Ok(Author {
            id: get_value_str(author, "id")?,
            name: get_value_str(author, "name")?,
            avatar_url: get_value_str(author, "avatar_url")?,
            email: get_value_str(&author["person"], "email")?,
            user_type: get_value_str(author, "type")?,
        })
    }
}


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
    pub content: Vec<Block>
}

impl Page {
    pub fn new(page: &Json) -> Result<Self> {
        let property_list = page.get("properties").expect("Page:::new() -> Unsupport Pages Format!");

        let author = Author::new(property_list)?;

        let mut properties: Vec<Property> = Vec::new();
        for (key, value) in property_list.as_object().unwrap().iter() {
            match key.as_str() {
                "Author" | "Created time" | "Edited time" | "Name" => (),
                _ => properties.push(Property::new(key, value)),
            }
        }

        Ok(Page {
            id: get_value_str(page, "id")?,
            created_time: get_value_str(page, "created_time")?,
            edited_time: get_value_str(page, "last_edited_time")?,
            author,
            editor_id: get_value_str(&page["last_edited_by"], "id")?,
            cover: get_value_str(page, "cover")?,
            icon: get_value_str(page, "icon")?,
            title: get_value_str(get_property_value(property_list, Some("Name"))?.get(0).unwrap(), "plain_text")?,
            archived: page.get("archived").ok_or(anyhow!(""))?.as_bool().ok_or(anyhow!(""))?,
            url: get_value_str(page, "url")?,
            properties,
            content: Vec::new(),
        })
    }

//     pub fn from_remote(key: String, id: String) -> Self {
//     }

    pub fn content(&mut self) -> Result<String> {
        let response = Request::new(Module::Blocks(self.id.to_string()).path())?.request(RequestMethod::GET, Json::default())?;
        for val in response["results"].as_array().unwrap().iter() {
            self.content.push(Block::new(val).unwrap());
        }

        let mut content = String::default();
        for line in self.content.iter() {
            content = content.trim_end().to_string() + "\n" + &line.to_string();
        }
        Ok(content.trim().to_string())
    }
}

impl ImpRequest for Page {
    fn search(id: String, body: Json) -> Result<Self> {
        let page = Request::new(Module::Pages(id).path())?.request(RequestMethod::GET, body)?;
        Page::new(&page)
    }
}
