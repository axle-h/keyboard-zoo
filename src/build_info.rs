use titlecase::titlecase;

pub const BUILD_INFO: &str = build_info::format!(
    "{} v{} by {}",
    $.crate_info.name,
    $.crate_info.version,
    // $.version_control.unwrap().git().unwrap().commit_short_id,
    $.crate_info.authors,
);

pub const APP_NAME: &str = build_info::format!("{}", $.crate_info.name);

pub fn nice_app_name() -> String {
    titlecase(&APP_NAME.replace("-", " "))
}
