use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "typecho_relationships")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub cid: u32,
    #[sea_orm(primary_key)]
    pub mid: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}