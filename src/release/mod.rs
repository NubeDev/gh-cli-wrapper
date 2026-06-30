//! Common `gh release` commands.

pub mod create;
pub mod delete;
pub mod download;
pub mod list;
pub mod view;

pub use create::ReleaseCreate;
pub use delete::ReleaseDelete;
pub use download::ReleaseDownload;
pub use list::ReleaseList;
pub use view::ReleaseView;
