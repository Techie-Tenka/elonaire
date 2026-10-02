use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::data::context::auth::use_auth;

/// Higher-order component that renders `children` if the user is
/// authenticated, or redirects to `/sign-in` once the auth state is known.
///
/// While the session is being bootstrapped (i.e. `auth_resolved()` is still
/// `false` — a `checkAuth` call is in flight, or hasn't started yet), this
/// renders nothing rather than redirecting. That's what prevents a hard
/// refresh from bouncing an authenticated user to the sign-in page.
///
/// Requires `AuthContext::bootstrap_session` to have been called once at
/// app root; otherwise `auth_resolved()` never flips and this will render
/// nothing forever.
///
/// Example usage:
/// ```
/// <Route path=StaticPathSegment("") view=|| view! { <ProtectedRoute><Home /></ProtectedRoute> } />
/// ```
#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    // Only navigate once we *know* the answer. Runs whenever `resolved` or
    // `is_authenticated` changes; if the user later signs out, this fires
    // again and redirects.
    Effect::new(move |_| {
        if !auth.is_loading.get() && !auth.is_authenticated().get() {
            navigate("/sign-in", Default::default());
        }
    });

    view! {
        {move || {
            if auth.is_loading.get() {
                // Still waiting on checkAuth — render nothing.
                // Swap for a spinner component if you have one.
                None
            } else if auth.is_authenticated().get() {
                Some(children().into_view())
            } else {
                // Redirect is in flight; render nothing to avoid a flash.
                None
            }
        }}
    }
    .into_any()
}
