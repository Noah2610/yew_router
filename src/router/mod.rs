//! Router component and related types.
pub mod router;

pub use router::{Props, Router, Render};

use crate::agent::AgentState;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait RouterState<'de>: AgentState<'de> + PartialEq {}

impl<'de, T> RouterState<'de> for T where T: AgentState<'de> + PartialEq {}
