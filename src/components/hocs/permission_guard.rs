use leptos::prelude::*;

use crate::data::context::auth::use_auth;

#[derive(Clone, Debug, PartialEq)]
pub enum PermissionMatch {
    /// User must have ALL listed permissions
    All,
    /// User must have AT LEAST ONE of the listed permissions
    Any,
}

/// Require a single permission
/// ```
/// <PermissionGuard permissions=vec!["read:blog_post".to_string()]>
///    <BlogPostList />
/// </PermissionGuard>
/// ```
///
/// Require ALL permissions
/// ```
/// <PermissionGuard
///    permissions=vec!["write:role".to_string(), "read:resource".to_string()]
///    match_mode=PermissionMatch::All
/// >
///    <AdminPanel />
/// </PermissionGuard>
/// ```
///
/// Require ANY permission
/// ```
/// <PermissionGuard
///    permissions=vec!["write:blog_post".to_string(), "write:role".to_string()]
///    match_mode=PermissionMatch::Any
/// >
///    <WriteButton />
/// </PermissionGuard>
/// ```

#[component]
pub fn PermissionGuard(
    /// Permissions required to render children
    #[prop(into)]
    permissions: Vec<String>,
    /// Match mode — All or Any
    #[prop(default = PermissionMatch::All, optional)]
    match_mode: PermissionMatch,
    /// Content to render if authorized
    children: ChildrenFn,
) -> impl IntoView {
    let auth_ctx = use_auth();

    let is_authorized = Memo::new(move |_| {
        let user_permissions = auth_ctx.current_role_permissions.get();

        match match_mode {
            PermissionMatch::All => permissions.iter().all(|p| user_permissions.contains(p)),
            PermissionMatch::Any => permissions.iter().any(|p| user_permissions.contains(p)),
        }
    });

    view! {
        <Show when=move || is_authorized.get() fallback=|| ()>
            {children()}
        </Show>
    }
    .into_any()
}
