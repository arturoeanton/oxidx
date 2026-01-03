use crate::schema::WindowSchema;
use anyhow::Result;

pub trait CodeGenerator {
    fn generate(&self, schema: &WindowSchema) -> Result<String>;
}
