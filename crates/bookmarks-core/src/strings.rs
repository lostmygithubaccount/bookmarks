//! Shared string constants used across CLI, app, and webapp.

// -- Project ----------------------------------------------------------------

pub const PROJECT_URL: &str = "https://dkdc.io/bookmarks/";

// -- Placeholders ------------------------------------------------------------

pub const PH_LINK_NAME: &str = "link name";
pub const PH_LINK_URL: &str = "https://...";
pub const PH_ALIAS_NAME: &str = "alias name";
pub const PH_ALIAS_TARGET: &str = "link name";
pub const PH_GROUP_NAME: &str = "group name";
pub const PH_GROUP_ENTRIES: &str = "link name, alias name, ...";
pub const PH_FILTER: &str = "filter...";

// -- Error templates ---------------------------------------------------------

pub fn err_alias_target_missing(target: &str) -> String {
    format!("alias target '{target}' does not exist in links")
}

pub fn err_group_entries_missing(missing: &[&str]) -> String {
    format!("group entries not found: {}", missing.join(", "))
}
