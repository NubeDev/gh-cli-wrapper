//! Common `gh pr` commands.

pub mod checkout;
pub mod close;
pub mod create;
pub mod list;
pub mod merge;
pub mod reopen;
pub mod review;
pub mod view;

pub use checkout::PrCheckout;
pub use close::PrClose;
pub use create::PrCreate;
pub use list::{PrList, PrState};
pub use merge::{PrMerge, PrMergeMethod};
pub use reopen::PrReopen;
pub use review::{PrReview, PrReviewAction};
pub use view::PrView;
