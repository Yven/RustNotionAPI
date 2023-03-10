pub mod page;
pub mod database;
pub mod sort;
pub mod filter;
pub mod property;
pub mod block;
pub mod request;
pub mod text;


use self::request::{Request, RequestMethod};
use sort::{Sort, Direction};
use filter::Filter;
use property::PropertyType;
pub use super::error::CommErr;

use std::fmt::Display;
pub use serde_json::Value as Json;
use anyhow::Result;


pub trait NewImp {
    fn new(val: &Json) -> Result<Self>  where Self: Sized;
    fn next(&self) -> Result<Self>  where Self: Sized {
        Err(CommErr::CErr("Do not have next page").into())
    }
    // fn search(builder: &NotionBuilder) -> Result<Self>  where Self: Sized;
}


#[allow(dead_code)]
pub enum Notion {
    Databases(String),
    Pages(String),
    Blocks(String),
    Users(String),
}

impl Notion {
    pub fn path(&self) -> String {
        {
            use Notion::*;
            match self {
                Databases(id) => "databases/".to_string() + &id + "/query",
                Pages(id) => "pages/".to_string() + &id,
                Blocks(id) => "blocks/".to_string() + &id + "/children",
                Users(id) => "users/".to_string() + &id,
            }
        }
    }

    pub fn method(&self) -> RequestMethod {
        {
            use Notion::*;
            match self {
                Databases(_) => RequestMethod::POST,
                Pages(_) => RequestMethod::GET,
                Blocks(_) => RequestMethod::GET,
                Users(_) => RequestMethod::GET,
            }
        }
    }

    pub fn get_val(&self) -> String {
        {
            use Notion::*;
            match self {
                Databases(s) |
                Pages(s) |
                Blocks(s) |
                Users(s) => s.to_string()
            }
        }
    }

    pub fn filter(self, filter: Filter) -> NotionBuilder {
        NotionBuilder::new(self).filter(filter)
    }

    pub fn sort(self, field: PropertyType, order: Direction) -> NotionBuilder  {
        NotionBuilder::new(self).sort(field, order)
    }

    pub fn limit(mut self, page_num: i32) -> NotionBuilder {
        NotionBuilder::new(self).limit(page_num)
    }

    pub fn search<T: NewImp>(self) -> Result<T> {
        let builder = NotionBuilder::new(self);
        let res = builder.request.query(builder.module.method(), builder.module.path(), builder.format_body())?;
        T::new(&res)
    }
}


const DEFAULT_PAGE_SIZE: i32 = 100;
pub struct NotionBuilder {
    pub module: Notion,
    request: Request,
    filter: Filter,
    sort: Sort,
    start_cursor: String,
    page_size: i32,
}

impl NotionBuilder {
    pub fn new(module: Notion) -> Self {
        let request = match Request::new() {
            Ok(s) => s,
            Err(e) => panic!("{}", e)
        };
        NotionBuilder {
            module,
            request,
            filter: Filter::default(),
            sort: Sort::default(),
            start_cursor: String::default(),
            page_size: DEFAULT_PAGE_SIZE,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        if self.filter.property.get_val().is_empty() {
            self.filter = filter;
        } else {
            self.filter = self.filter.and(filter);
        }

        self
    }

    pub fn sort(mut self, field: PropertyType, order: Direction) -> Self {
        self.sort.add(vec![(field, order)]);
        self
    }

    // pub fn find(&self) -> T {
    // }

    pub fn limit(mut self, size: i32) -> Self {
        self.page_size = size;
        self
    }

    pub fn search<T: NewImp>(&self) -> Result<T> {
        let res = self.request.query(self.module.method(), self.module.path(), self.format_body())?;
        T::new(&res)
    }

    pub fn format_body(&self) -> Json {
        serde_json::from_str::<Json>(&self.to_string()).unwrap()
    }
}

impl Display for NotionBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut body: Vec<String> = Vec::new();

        if !self.start_cursor.is_empty() {
            body.push(format!(r#""start_cursor": "{}""#, self.start_cursor));
        }

        let filter = self.filter.to_string();
        if !filter.is_empty() {
            body.push(format!(r#""filter": {}"#, filter));
        }

        let sort = self.sort.to_string();
        if !sort.is_empty() {
            body.push(format!(r#""sorts": {}"#, sort));
        }

        body.push(format!(r#""page_size": {}"#, self.page_size));

        write!(f, r#"{{{}}}"#, body.join(","))
    }
}


/**
 * 获取Notion属性数组中的属性值
 */
fn get_property_value<'a>(property: &'a Json, index: Option<&'static str>) -> Result<&'a Json> {
    let property = match index {
        Some(i) => &property.get(i).ok_or(CommErr::FormatErr(index.unwrap()))?,
        None => property,
    };

    property.get(get_value_str(property, "type")?).ok_or(CommErr::FormatErr("type").into())
}

/**
 * 获取Json中的某个值的String形式
 */
fn get_value_str(value: &Json, index: &'static str) -> Result<String> {
    Ok(
        value.get(index).ok_or(CommErr::FormatErr(index))?
            .as_str().ok_or(CommErr::GetValueStrErr(index))?
            .to_string()
    )
}