use leptos::prelude::*;

use crate::components::hocs::permission_guard::PermissionMatch;
use crate::data::context::auth::use_auth;

/// Usage in a component
/// ```
/// let can_write = use_permission(
///    vec!["write:blog_post".to_string()],
///    PermissionMatch::All,
/// );
/// ```
///
/// Use in view
/// ```
/// <BasicButton
///    disabled=Memo::new(move |_| !can_write.get())
///    button_text="Publish"
/// />
/// ```
///
/// Use in event handler
/// ```
/// let handle_click = move |_| {
///    if !can_write.get_untracked() {
///        return;
///    }
///    // proceed
/// };
/// ```

pub fn use_permission(permissions: &[String], match_mode: PermissionMatch) -> Memo<bool> {
    let auth_ctx = use_auth();
    // Clone once here at the boundary, not at every call site
    let permissions = StoredValue::new(permissions.to_vec());
    let match_mode = StoredValue::new(match_mode);

    Memo::new(move |_| {
        let user_permissions = auth_ctx.current_role_permissions.get();
        let permissions = permissions.get_value();
        match match_mode.get_value() {
            PermissionMatch::All => permissions.iter().all(|p| user_permissions.contains(p)),
            PermissionMatch::Any => permissions.iter().any(|p| user_permissions.contains(p)),
        }
    })
}
