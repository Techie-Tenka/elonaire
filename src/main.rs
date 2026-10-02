mod app;
mod components {
    pub mod hocs {
        pub mod permission_guard;
        pub mod protected_route;
    }
    pub mod molecules {
        pub mod blog {
            pub mod author_info;
            pub mod blog_comment;
            pub mod blog_post;
            pub mod blog_post_metadata;
            pub mod blog_section;
            pub mod featured_post;
        }
        pub mod cookie_banner;
        pub mod flip_card;
        pub mod footer;
        pub mod headline;
        pub mod nav;
        pub mod quick_action;
        pub mod ratecard;
        pub mod section_title;
        pub mod stats_card;
        pub mod top_nav;
        pub mod auth {
            pub mod auth_modal;
            pub mod sign_in_form;
            pub mod sign_up_form;
        }
        pub mod skeleton;
    }
}
mod data {
    pub mod context {
        pub mod acl;
        pub mod auth;
        pub mod billing;
        pub mod blog;
        pub mod portfolio;
        pub mod shared;
        pub mod site_owner;
        pub mod ui;
        pub mod user;
    }
    pub mod models {
        pub mod general {
            pub mod acl;
            pub mod files;
            pub mod shared;
        }
        pub mod graphql {
            pub mod acl;
            pub mod email;
            pub mod shared;
        }
    }
}
mod utils {
    pub mod hooks {
        pub mod use_permissions;
    }
    pub mod custom_traits;
    pub mod errors;
    pub mod graphql_client;
}
mod views {
    pub mod dashboard {
        pub mod blog;
        pub mod departments;
        pub mod home;
        pub mod layout;
        pub mod organizations;
        pub mod permissions;
        pub mod portfolio;
        pub mod professional_details;
        pub mod ratecards;
        pub mod resources;
        pub mod resume;
        pub mod roles;
        pub mod service_rates;
        pub mod service_requests;
        pub mod skills;
        pub mod user_profile;
        pub mod user_services;
        pub mod users;
    }
    pub mod public {
        pub mod blog {
            pub mod about;
            pub mod blog_post_detail;
            pub mod home;
            pub mod layout;
        }
        pub mod about;
        pub mod attributions;
        pub mod contact;
        pub mod error_handler;
        pub mod errors;
        pub mod faqs;
        pub mod home;
        pub mod layout;
        pub mod login;
        pub mod portfolio;
        pub mod privacy;
        pub mod ratecard;
        pub mod resume;
        pub mod sign_up;
        pub mod tos;
        pub mod waitlist;
    }
}

use app::*;

fn main() {
    leptos::mount::mount_to_body(App)
}
