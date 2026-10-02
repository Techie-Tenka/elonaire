use detaxine_ui::stacks::z_stack::provide_z_stack;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
    path,
};

use crate::{
    components::{hocs::protected_route::ProtectedRoute, molecules::cookie_banner::CookieBanner},
    data::context::{
        acl::provide_acl, auth::provide_auth, billing::provide_billing, blog::provide_blog,
        portfolio::provide_portfolio, shared::provide_shared, site_owner::provide_site_owner,
        ui::provide_ui, user::provide_user,
    },
    views::{
        dashboard::{
            blog::{Blog, BlogList, CreateBlog},
            departments::{CreateDepartment, Departments, DepartmentsList},
            home::DashboardHome,
            layout::DashboardLayout,
            organizations::{CreateOrganization, Organizations, OrganizationsList},
            permissions::{CreatePermission, Permissions, PermissionsList},
            portfolio::{CreatePortfolio, Portfolio, PortfolioList},
            professional_details::{
                CreateProfessionalDetail, ProfessionalDetails, ProfessionalDetailsList,
            },
            ratecards::{CreateRatecard, Ratecards, RatecardsList},
            resources::{CreateResource, Resources, ResourcesList},
            resume::{CreateResumeItem, Resume, ResumeItemsList},
            roles::{CreateRole, Roles, RolesList},
            service_rates::{CreateServiceRate, ServiceRates, ServiceRatesList},
            service_requests::{ServiceRequests, ServiceRequestsList},
            skills::{CreateSkill, Skills, SkillsList},
            user_profile::ProfilePage,
            user_services::{CreateUserService, UserService, UserServicesList},
            users::{CreateUser, Users, UsersList},
        },
        public::{
            about::About,
            attributions::Attributions,
            blog::{
                about::About as AboutBlog, blog_post_detail::BlogPostDetail, home::BlogHome,
                layout::BlogLayout,
            },
            contact::Contact,
            errors::*,
            faqs::Faqs,
            home::Home,
            layout::MainLayout,
            login::SignIn,
            portfolio::Portfolio as PublicPortfolio,
            privacy::PrivacyPolicy,
            ratecard::Ratecard as PublicRatecard,
            resume::Resume as PublicResume,
            sign_up::SignUp,
            tos::TermsOfService,
            waitlist::WaitList,
        },
    },
};

#[component]
pub fn App() -> impl IntoView {
    // ── Context wiring ────────────────────────────────────────────────
    // Each `provide_*()` returns its own context by value (all contexts
    // are `Copy`), so downstream providers can take a `&` to a dep and
    // store a copy without any lifetime or ownership gymnastics.
    let ui = provide_ui();

    let auth = provide_auth(ui);

    let user = provide_user(ui, auth);
    let site_owner = provide_site_owner(ui, auth);
    let acl = provide_acl(ui, auth);
    let portfolio = provide_portfolio(ui, auth);
    let billing = provide_billing(ui, auth);
    let blog = provide_blog(ui, user, auth);
    provide_shared(ui, auth);

    // The `provide_*()` side effect (stashing into the context map) is what
    // matters. Silence unused-variable warnings for the ones App itself
    // doesn't reference.
    let _ = (site_owner, acl, portfolio, billing, blog);

    provide_meta_context();
    provide_z_stack();

    // ── Auth bootstrap ────────────────────────────────────────────────
    // `auth` is captured by value into the closure (Copy). Reading the
    // token here is safe because App's owner already ran `provide_auth`.
    // The gate on empty-token preserves the original behaviour of not
    // firing checkAuth on a fresh, unauthenticated load.
    Effect::new(move |_| {
        // calls into `AuthContext::authenticate_with_token` →
        // `UserContext::fetch_own_profile_by_id` via the context's own
        // user_id watcher (or the util, if you kept it).
        auth.authenticate_with_token();
        user.fetch_own_profile();
    });

    view! {
        <Link rel="apple-touch-icon" sizes="180x180" href="public/apple-touch-icon.png"/>
        <Link rel="icon" type_="image/png" sizes="32x32" href="public/favicon-32x32.png"/>
        <Link rel="icon" type_="image/png" sizes="16x16" href="public/favicon-16x16.png"/>
        <Link rel="manifest" href="public/site.webmanifest"/>
        <Stylesheet id="leptos" href="/style/output.css"/>
        <CookieBanner />
        <div id="modal-root"></div>
        <ErrorBoundary
                        // the fallback receives a signal containing current errors
                        fallback=|errors| view! {
                            <div class="error">
                                <p>"Something went wrong: "</p>
                                // we can render a list of errors
                                // as strings, if we'd like
                                // I might improve this one
                                <ul>
                                    {move || errors.get()
                                        .into_iter()
                                        .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                                        .collect::<Vec<_>>()
                                    }
                                </ul>
                            </div>
                        }
                    >
            <Router>
                <Routes fallback=NotFound>
                    // <Route path=StaticSegment("") view=Home />
                    <ParentRoute path=path!("") view=MainLayout>
                        <Route path=path!("/faq") view=Faqs />
                        <Route path=path!("/terms") view=TermsOfService />
                        <Route path=path!("/privacy") view=PrivacyPolicy />
                        <Route path=path!("/contact") view=Contact />
                        <Route path=path!("/attributions") view=Attributions />
                        <Route path=path!("/marketplace") view=WaitList />
                        <Route path=path!("") view=Home />
                    </ParentRoute>
                    <Route path=StaticSegment("/about") view=About />
                    <Route path=StaticSegment("/resume") view=PublicResume />
                    <Route path=StaticSegment("/portfolio") view=PublicPortfolio />
                    <Route path=StaticSegment("/ratecard") view=PublicRatecard />
                    <ParentRoute path=path!("/dashboard") view=|| view! { <ProtectedRoute><DashboardLayout /></ProtectedRoute> }>
                        <ParentRoute path=path!("/portfolio") view=Portfolio>
                            <Route path=path!("") view=PortfolioList />
                            <Route path=path!("create") view=CreatePortfolio />
                        </ParentRoute>
                        <ParentRoute path=path!("/professional-details") view=ProfessionalDetails>
                            <Route path=path!("") view=ProfessionalDetailsList />
                            <Route path=path!("create") view=CreateProfessionalDetail />
                        </ParentRoute>
                        <ParentRoute path=path!("/services") view=UserService>
                            <Route path=path!("") view=UserServicesList />
                            <Route path=path!("create") view=CreateUserService />
                        </ParentRoute>
                        <ParentRoute path=path!("/service-rates") view=ServiceRates>
                            <Route path=path!("") view=ServiceRatesList />
                            <Route path=path!("create") view=CreateServiceRate />
                        </ParentRoute>
                        <ParentRoute path=path!("/service-requests") view=ServiceRequests>
                            <Route path=path!("") view=ServiceRequestsList />
                        </ParentRoute>
                        <ParentRoute path=path!("/ratecards") view=Ratecards>
                            <Route path=path!("") view=RatecardsList />
                            <Route path=path!("create") view=CreateRatecard />
                        </ParentRoute>
                        <ParentRoute path=path!("/resume") view=Resume>
                            <Route path=path!("") view=ResumeItemsList />
                            <Route path=path!("create") view=CreateResumeItem />
                        </ParentRoute>
                        <ParentRoute path=path!("/skills") view=Skills>
                            <Route path=path!("") view=SkillsList />
                            <Route path=path!("create") view=CreateSkill />
                        </ParentRoute>
                        <ParentRoute path=path!("/blog") view=Blog>
                            <Route path=path!("") view=BlogList />
                            <Route path=path!("create") view=CreateBlog />
                        </ParentRoute>
                        <ParentRoute path=path!("/users") view=Users>
                            <Route path=path!("") view=UsersList />
                            <Route path=path!("create") view=CreateUser />
                        </ParentRoute>
                        <ParentRoute path=path!("/roles") view=Roles>
                            <Route path=path!("") view=RolesList />
                            <Route path=path!("create") view=CreateRole />
                        </ParentRoute>
                        <ParentRoute path=path!("/permissions") view=Permissions>
                            <Route path=path!("") view=PermissionsList />
                            <Route path=path!("create") view=CreatePermission />
                        </ParentRoute>
                        <ParentRoute path=path!("/resources") view=Resources>
                            <Route path=path!("") view=ResourcesList />
                            <Route path=path!("create") view=CreateResource />
                        </ParentRoute>
                        <ParentRoute path=path!("/organizations") view=Organizations>
                            <Route path=path!("") view=OrganizationsList />
                            <Route path=path!("create") view=CreateOrganization />
                        </ParentRoute>
                        <ParentRoute path=path!("/departments") view=Departments>
                            <Route path=path!("") view=DepartmentsList />
                            <Route path=path!("create") view=CreateDepartment />
                        </ParentRoute>
                        <Route path=path!("/user/profile") view=ProfilePage />
                        <Route path=path!("") view=DashboardHome />
                    </ParentRoute>
                    <ParentRoute path=path!("/blog") view=BlogLayout >
                        <Route path=path!("") view=BlogHome />
                        <Route path=path!("/read/:slug") view=BlogPostDetail />
                        <Route path=path!("/about") view=AboutBlog />
                    </ParentRoute>
                    <Route path=StaticSegment("/500") view=InternalServerError/>
                    <Route path=StaticSegment("/sign-in") view=SignIn/>
                    <Route path=StaticSegment("/sign-up") view=SignUp/>
                </Routes>
            </Router>
        </ErrorBoundary>
    }.into_any()
}
