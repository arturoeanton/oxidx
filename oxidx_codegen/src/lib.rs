pub mod languages;
pub mod schema;
pub mod traits;
pub mod view_generator;

pub use languages::rust::RustGenerator;
pub use schema::{generate_json_schema, ComponentNode, WindowSchema};
pub use traits::CodeGenerator;
pub use view_generator::generate_view;

#[cfg(test)]
mod tests;
