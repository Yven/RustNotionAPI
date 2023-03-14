pub mod contents;
pub mod metas;
pub mod relationships;

use sea_orm::{TransactionTrait, DatabaseConnection, ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter};
use crate::error::CommErr;

use super::notion::page;
use chrono::DateTime;
use anyhow::Result;


pub async fn is_exist(db: &DatabaseConnection, slug: String) -> Result<bool> {
    let model = contents::Entity::find().filter(contents::Column::Slug.eq(Some(slug))).one(db).await?;
    match model {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub async fn new_article(db: &DatabaseConnection, page: page::Page) -> Result<()> {
    db.transaction::<_, (), CommErr>(|txn| {
        Box::pin(async move {
            let content_res = contents::ActiveModel {
                title: Set(Some(page.title.clone())),
                slug: Set(Some(page.search_property("Slug")?[0].0.clone())),
                created: Set(DateTime::parse_from_rfc3339(&page.created_time)?.timestamp() as u32),
                modified: Set(DateTime::parse_from_rfc3339(&page.edited_time)?.timestamp() as u32),
                text: Set(format!("<!--markdown-->{}", page.content.to_string())),
                author_id: Set(1),
                ctype: Set("post".to_owned()),
                status: Set("publish".to_owned()),
                allow_comment: Set("1".to_owned()),
                allow_ping: Set("0".to_owned()),
                allow_feed: Set("1".to_owned()),
                ..Default::default()
            }.insert(txn)
            .await?;

            let tag_list = page.search_property("Tag")?;
            let mut noexist_tag_list = Vec::new();
            for tag in tag_list.iter() {
                let metas_model = metas::Entity::find().filter(metas::Column::Name.eq(tag.0.clone())).one(txn).await?;

                match metas_model {
                    Some(model) => {
                        let count = model.count;
                        let mut model: metas::ActiveModel = model.into();
                        model.count = Set(count + 1);
                        model.update(txn).await?;
                    },
                    None => noexist_tag_list.push((tag.0.as_str(), tag.1.as_str(), "tag")),
                }
            }

            let category = page.search_property("Category")?;
            let metas_model = metas::Entity::find().filter(metas::Column::Name.eq(category[0].0.clone())).one(txn).await?;
            match metas_model {
                Some(model) => {
                    let count = model.count;
                    let mut model: metas::ActiveModel = model.into();
                    model.count = Set(count + 1);
                    model.update(txn).await?;
                },
                None => noexist_tag_list.push((category[0].0.as_str(), category[0].1.as_str(), "category")),
            }

            for tag in noexist_tag_list {
                let metas_res = metas::ActiveModel {
                    name: Set(Some(tag.0.to_string())),
                    slug: Set(Some(tag.1.to_string())),
                    mtype: Set(Some(tag.2.to_string())),
                    count: Set(1),
                    order: Set(0),
                    parent: Set(0),
                    ..Default::default()
                }.insert(txn)
                .await?;

                relationships::ActiveModel {
                    cid: Set(content_res.cid),
                    mid: Set(metas_res.mid)
                }.insert(txn)
                .await?;
            }

            Ok(())
        })
    })
    .await?;

    Ok(())
}

pub async fn update_article(db: &DatabaseConnection, page: page::Page) -> Result<()> {
    db.transaction::<_, (), CommErr>(|txn| {
        Box::pin(async move {
            let slug = page.search_property("slug")?;
            let model = contents::Entity::find().filter(contents::Column::Slug.eq(Some(slug[0].0.clone()))).one(txn).await?.ok_or(CommErr::CErr("page do not exist"))?;

            let mut model: self::contents::ActiveModel = model.into();
            model.title = Set(Some(page.title.clone()));
            model.modified = Set(DateTime::parse_from_rfc3339(&page.edited_time)?.timestamp() as u32);
            model.text = Set(format!("<!--markdown-->{}", page.content.to_string()));
            model.update(txn).await?;

            Ok(())
        })
    })
    .await?;

    Ok(())
}