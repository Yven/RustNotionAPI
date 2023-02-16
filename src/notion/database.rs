use super::{Request, page::Page, term::ReqBody, NotionModule};
use super::CONFIG_MAP;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Database {
    pub page_list: Vec<Page>
    // TODO: next_cursor/has_more/type/page属性
}

impl Database {
    // pub fn new(list: &Vec<Value>) -> Self {
    pub fn from_remote(id: &str, body: ReqBody) -> Self {
        let key = CONFIG_MAP.get("key").unwrap();
        let request = Request::new(key);
        let response = request.query(NotionModule::Databases, id, body).unwrap();
        let list = response["results"].as_array().unwrap();

        let mut page_list = Vec::new();
        for page in list.iter() {
            page_list.push(Page::new(page));
        }

        Database { page_list }
    }
}