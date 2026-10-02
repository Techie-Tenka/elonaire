use std::collections::HashMap;

use detaxine_ui::utils::forms::{FormDeserializeOptions, deserialize_form_data_with_options};
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::{File, FormData};

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext, user::UserContext},
        models::{
            general::{files::UploadedFileResponse, shared::RestResponse},
            graphql::{
                acl::User,
                email::{
                    CreateSubscriptionResponse, CreateSubscriptionVars, SubscriberInput,
                    SubscriptionInput, SubscriptionInputMetadata,
                },
                shared::{
                    BlogCommentInput, BlogPost, BlogPostInput, BlogStatus,
                    BookmarkBlogPostResponse, BookmarkBlogPostVars, CreateBlogCommentResponse,
                    CreateBlogCommentVars, CreateBlogPostResponse, CreateBlogPostVars,
                    FetchBlogPostsQueryFilters, FetchBlogPostsResponse, FetchBlogPostsVars,
                    FetchSingleBlogPostResponse, FetchSingleBlogPostVars,
                    ReactToBlogCommentResponse, ReactToBlogCommentVars, ReactToBlogPostResponse,
                    ReactToBlogPostVars, ReactionInput, ReactionType,
                    UpdateBlogPostShareCountResponse, UpdateBlogPostShareCountVars,
                },
            },
        },
    },
    utils::{
        custom_traits::IntoGlooHeaders,
        errors::{handle_graphql_errors, unwrap_rest_response},
        graphql_client::perform_mutation_or_query_with_vars,
    },
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");
const FILES_SERVICE_API: Option<&str> = option_env!("FILES_SERVICE_API");
const EMAIL_SERVICE_API: Option<&str> = option_env!("EMAIL_SERVICE_API");
const NEWSLETTER_MAILING_LIST_ID: Option<&str> = option_env!("NEWSLETTER_MAILING_LIST_ID");

const FEATURED_POSTS_QUERY: &str = r#"
    query FetchBlogPosts($filters: FetchBlogPostsQueryFilters) {
        fetchBlogPosts(filters: $filters) {
            data {
                title shortDescription status thumbnail category link
                publishedDate isFeatured isPremium createdAt updatedAt
                id author readTime
            }
            metadata { requestId newAccessToken }
        }
    }
"#;

const OTHER_POSTS_QUERY: &str = r#"
    query FetchBlogPosts($filters: FetchBlogPostsQueryFilters) {
        fetchBlogPosts(filters: $filters) {
            data {
                title shortDescription status thumbnail category link
                publishedDate isFeatured isPremium createdAt updatedAt
                id author readTime
            }
            metadata { requestId newAccessToken }
        }
    }
"#;

const SEARCH_POSTS_QUERY: &str = r#"
    query FetchBlogPosts($filters: FetchBlogPostsQueryFilters) {
        fetchBlogPosts(filters: $filters) {
            data { title category link }
            metadata { requestId newAccessToken }
        }
    }
"#;

// ── File upload helper ──────────────────────────────────────────────────

/// Uploads the thumbnail to the FILES service and returns its view URL.
/// Returns `None` on failure, having logged the underlying error and
/// surfaced a user-visible message via `ui`.
async fn upload_blog_thumbnail(
    files: &[File],
    auth: &AuthContext,
    ui: &UiContext,
) -> Option<String> {
    let Ok(files_form_data) = FormData::new() else {
        leptos::logging::error!("Failed to construct FormData for upload");
        ui.set_client_error("Could not prepare the file upload.");
        return None;
    };

    for file in files {
        if let Err(e) = files_form_data.append_with_blob("thumbnail", file) {
            leptos::logging::error!("Failed to append Blob: {:?}", e);
            ui.set_client_error("Could not attach a file to the upload.");
            return None;
        }
    }

    let Some(files_service_api) = FILES_SERVICE_API else {
        leptos::logging::error!("FILES_SERVICE_API is not configured");
        ui.set_client_error("File service is not configured.");
        return None;
    };

    let Ok(request) = gloo_net::http::Request::post(&format!("{files_service_api}/upload/default"))
        .headers(auth.headers().into_gloo_headers())
        .body(files_form_data)
    else {
        leptos::logging::error!("Failed to build upload request");
        ui.set_client_error("Could not build the upload request.");
        return None;
    };

    let response = match request.send().await {
        Ok(r) => r,
        Err(err) => {
            leptos::logging::error!("Failed to upload files: {:?}", err);
            ui.set_client_error("Upload failed. Please check your connection and try again.");
            return None;
        }
    };

    let body = match response
        .json::<RestResponse<Vec<UploadedFileResponse>>>()
        .await
    {
        Ok(b) => b,
        Err(err) => {
            leptos::logging::error!("Failed to parse upload response: {:?}", err);
            ui.set_client_error("Unexpected response from the file service.");
            return None;
        }
    };

    // `unwrap_rest_response` surfaces the API-level message via `ui` itself.
    let uploaded_files = unwrap_rest_response(body, ui, auth, None)?;

    let Some(thumbnail) = uploaded_files
        .into_iter()
        .find(|f| f.field_name == "thumbnail")
    else {
        leptos::logging::error!("Upload response contained no thumbnail file");
        ui.set_client_error("Upload succeeded but no thumbnail was returned.");
        return None;
    };

    Some(format!(
        "{files_service_api}/view/default/{}",
        thumbnail.original_filename
    ))
}

// ── Context ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Copy)]
pub struct BlogContext {
    ui: UiContext,
    user: UserContext,
    auth: AuthContext,
    pub blog_posts: RwSignal<Vec<BlogPost>>,
    pub current_blog_post: RwSignal<Option<BlogPost>>,
    pub is_loading: RwSignal<bool>,
    pub created_post_dirty: RwSignal<u64>,
    pub comment_created_dirty: RwSignal<u64>,
    pub post_reaction_dirty: RwSignal<u64>,
    pub comment_reaction_dirty: RwSignal<u64>,
    pub share_count_updated_dirty: RwSignal<u64>,
    pub bookmark_toggled_dirty: RwSignal<u64>,
    pub featured_posts: RwSignal<Vec<BlogPost>>,
    pub other_posts: RwSignal<Vec<BlogPost>>,
    pub search_results: RwSignal<Vec<BlogPost>>,
    pub search_is_loading: RwSignal<bool>,
    pub subscription_created_dirty: RwSignal<u64>,
}

impl BlogContext {
    pub fn new(ui: UiContext, user: UserContext, auth: AuthContext) -> Self {
        Self {
            ui,
            user,
            auth,
            blog_posts: RwSignal::new(Vec::new()),
            current_blog_post: RwSignal::new(None),
            is_loading: RwSignal::new(false),
            created_post_dirty: RwSignal::new(0),
            comment_created_dirty: RwSignal::new(0),
            post_reaction_dirty: RwSignal::new(0),
            comment_reaction_dirty: RwSignal::new(0),
            share_count_updated_dirty: RwSignal::new(0),
            bookmark_toggled_dirty: RwSignal::new(0),
            featured_posts: RwSignal::new(Vec::new()),
            other_posts: RwSignal::new(Vec::new()),
            search_results: RwSignal::new(Vec::new()),
            search_is_loading: RwSignal::new(false),
            subscription_created_dirty: RwSignal::new(0),
        }
    }

    /// Query text is dynamic — selection set varies by call site — so it's
    /// still passed in rather than hardcoded, same as the original.
    pub fn fetch_blog_posts(&self, filters: FetchBlogPostsVars, query: &'static str) {
        let blog_posts_sig = self.blog_posts;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                FetchBlogPostsResponse,
                FetchBlogPostsVars,
            >(Some(&headers), shared_service_api, query, filters)
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_blog_posts
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    blog_posts_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_single_blog_post(&self, vars: FetchSingleBlogPostVars) {
        let current_blog_post_sig = self.current_blog_post;
        let loading = self.is_loading;
        let ui = self.ui;
        let user = self.user;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                        query FetchSingleBlogPost($blogIdOrSlug: String!) {
                            fetchSingleBlogPost(blogIdOrSlug: $blogIdOrSlug) {
                                data {
                                    title
                                    shortDescription
                                    status
                                    thumbnail
                                    category
                                    link
                                    publishedDate
                                    isFeatured
                                    isPremium
                                    createdAt
                                    updatedAt
                                    id
                                    author
                                    content
                                    readTime
                                    comments {
                                        content
                                        createdAt
                                        updatedAt
                                        id
                                        replyCount
                                        author
                                        reactionCount
                                        currentUserReaction {
                                            reactionType
                                            id
                                        }
                                    }
                                    reactionCount
                                    currentUserReaction {
                                        reactionType
                                        id
                                    }
                                    bookmarksCount
                                    sharesCount
                                    currentUserBookmarked
                                }
                                metadata {
                                    requestId
                                    newAccessToken
                                }
                            }
                        }
                       "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                FetchSingleBlogPostResponse,
                FetchSingleBlogPostVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            let Some(data) = response.get_data() else {
                handle_graphql_errors(&response, &ui, None);
                loading.set(false);
                return;
            };

            let mut blog_post = data
                .fetch_single_blog_post
                .as_ref()
                .unwrap_or(&Default::default())
                .get_data()
                .to_owned();

            // Hydrate authors — one request per unique user id, cached across
            // comments so a prolific commenter isn't fetched N times.
            let mut cache: HashMap<String, User> = HashMap::new();

            if let Some(comments) = &mut blog_post.comments {
                for comment in comments.iter_mut() {
                    if let Some(author_id) = comment.author.clone() {
                        if !cache.contains_key(&author_id) {
                            if let Some(u) = user.fetch_user_by_id_async(author_id.clone()).await {
                                cache.insert(author_id.clone(), u);
                            }
                        }
                        comment.full_author_details = cache.get(&author_id).cloned();
                    }
                }
            }

            if let Some(author_id) = blog_post.author.clone() {
                blog_post.full_author_details = match cache.get(&author_id) {
                    Some(u) => Some(u.clone()),
                    None => user.fetch_user_by_id_async(author_id).await,
                };
            }

            current_blog_post_sig.set(Some(blog_post));
            loading.set(false);
        });
    }

    pub fn create_blog_post(&self, thumbnail_files: Vec<File>, form_data: FormData) {
        let created_post_dirty = self.created_post_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(thumbnail_url) = upload_blog_thumbnail(&thumbnail_files, &auth, &ui).await
            else {
                loading.set(false);
                return;
            };

            if let Err(e) = form_data.append_with_str("thumbnail", &thumbnail_url) {
                leptos::logging::error!("Failed to append thumbnail to form data: {:?}", e);
                ui.set_client_error("Could not attach the uploaded file to the form.");
                loading.set(false);
                return;
            }

            let Some(deserialized) = deserialize_form_data_with_options::<BlogPostInput>(
                &form_data,
                &FormDeserializeOptions {
                    deserialize_bool: true,
                    ..Default::default()
                },
            ) else {
                loading.set(false);
                return;
            };

            let vars = CreateBlogPostVars {
                blog_post: deserialized,
            };

            let query = r#"
                    mutation CreateBlogPost($blogPost: BlogPostInput!) {
                        createBlogPost(blogPost: $blogPost) {
                            data {
                                id
                                shortDescription
                                title
                                status
                                category
                                link
                                thumbnail
                                publishedDate
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                    }
                   "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateBlogPostResponse,
                CreateBlogPostVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(_data) => {
                    created_post_dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn create_blog_comment(&self, blog_post_id: String, blog_comment: BlogCommentInput) {
        let current_post = self.current_blog_post;
        let loading = self.is_loading;
        let dirty = self.comment_created_dirty;
        let ui = self.ui;
        let user = self.user;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                mutation AddCommentToBlogPost($blogComment: BlogCommentInput!, $blogPostId: String!) {
                    addCommentToBlogPost(blogComment: $blogComment, blogPostId: $blogPostId) {
                        data { content createdAt updatedAt id replyCount author }
                        metadata { requestId newAccessToken }
                    }
                }
            "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let vars = CreateBlogCommentVars {
                blog_comment,
                blog_post_id,
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateBlogCommentResponse,
                CreateBlogCommentVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            let Some(data) = response.get_data() else {
                handle_graphql_errors(&response, &ui, None);
                loading.set(false);
                return;
            };

            let mut new_comment = data
                .add_comment_to_blog_post
                .as_ref()
                .unwrap_or(&Default::default())
                .get_data();

            if let Some(author_id) = new_comment.author.clone() {
                if let Some(u) = user.fetch_user_by_id_async(author_id).await {
                    new_comment.full_author_details = Some(u);
                }
            }

            current_post.update(|prev| {
                if let Some(prev) = prev {
                    prev.comments = Some(match prev.comments.take() {
                        Some(mut c) => {
                            c.push(new_comment);
                            c
                        }
                        None => vec![new_comment],
                    });
                }
            });

            dirty.update(|n| *n += 1);
            loading.set(false);
        });
    }

    pub fn react_to_blog_post(&self, blog_post_id: String, reaction: ReactionType) {
        let current_post = self.current_blog_post;
        let loading = self.is_loading;
        let dirty = self.post_reaction_dirty;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                mutation ReactToBlogPost($reaction: ReactionInput!, $blogPostId: String!) {
                    reactToBlogPost(reaction: $reaction, blogPostId: $blogPostId) {
                        data { reactionType id }
                        metadata { requestId newAccessToken }
                    }
                }
            "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let vars = ReactToBlogPostVars {
                reaction: ReactionInput {
                    reaction_type: reaction,
                },
                blog_post_id,
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                ReactToBlogPostResponse,
                ReactToBlogPostVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(data) => {
                    current_post.update(|prev| {
                        if let Some(prev) = prev {
                            if prev.current_user_reaction.is_none() {
                                prev.reaction_count = prev.reaction_count.map(|v| v + 1);
                            }
                            prev.current_user_reaction = Some(
                                data.react_to_blog_post
                                    .as_ref()
                                    .unwrap_or(&Default::default())
                                    .get_data(),
                            );
                        }
                    });
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn react_to_blog_comment(&self, comment_id: String, reaction: ReactionType) {
        let current_post = self.current_blog_post;
        let loading = self.is_loading;
        let dirty = self.comment_reaction_dirty;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                mutation ReactToBlogComment($reaction: ReactionInput!, $commentId: String!) {
                    reactToBlogComment(reaction: $reaction, commentId: $commentId) {
                        data { reactionType id }
                        metadata { requestId newAccessToken }
                    }
                }
            "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let vars = ReactToBlogCommentVars {
                reaction: ReactionInput {
                    reaction_type: reaction,
                },
                comment_id: comment_id.clone(),
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                ReactToBlogCommentResponse,
                ReactToBlogCommentVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(data) => {
                    current_post.update(|prev| {
                        if let Some(prev) = prev {
                            if let Some(comments) = prev.comments.as_mut() {
                                for comment in comments.iter_mut() {
                                    if comment.id.as_deref() == Some(comment_id.as_str()) {
                                        comment.current_user_reaction = data
                                            .react_to_blog_comment
                                            .as_ref()
                                            .map(|v| v.get_data());

                                        if comment.current_user_reaction.is_none() {
                                            comment.reaction_count =
                                                comment.reaction_count.map(|v| v + 1);
                                        }
                                    }
                                }
                            }
                        }
                    });
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn update_blog_post_share_count(&self, blog_post_id: String) {
        let current_post = self.current_blog_post;
        let loading = self.is_loading;
        let dirty = self.share_count_updated_dirty;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                mutation UpdateBlogPostShareCount($blogPostId: String!) {
                    updateBlogPostShareCount(blogPostId: $blogPostId) {
                        data
                        metadata { requestId newAccessToken }
                    }
                }
            "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let vars = UpdateBlogPostShareCountVars { blog_post_id };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                UpdateBlogPostShareCountResponse,
                UpdateBlogPostShareCountVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(data) => {
                    current_post.update(|prev| {
                        if let Some(prev) = prev {
                            prev.shares_count = Some(
                                data.update_blog_post_share_count
                                    .as_ref()
                                    .unwrap_or(&Default::default())
                                    .get_data(),
                            );
                        }
                    });
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn bookmark_blog_post(&self, blog_post_id: String) {
        let current_post = self.current_blog_post;
        let loading = self.is_loading;
        let dirty = self.bookmark_toggled_dirty;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                mutation BookmarkBlogPost($blogPostId: String!) {
                    bookmarkBlogPost(blogPostId: $blogPostId) {
                        data
                        metadata { requestId newAccessToken }
                    }
                }
            "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let vars = BookmarkBlogPostVars { blog_post_id };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                BookmarkBlogPostResponse,
                BookmarkBlogPostVars,
            >(Some(&headers), shared_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(data) => {
                    current_post.update(|prev| {
                        if let Some(prev) = prev {
                            let was_bookmarked = prev.current_user_bookmarked.unwrap_or(false);
                            if was_bookmarked {
                                prev.bookmarks_count = prev.bookmarks_count.map(|v| v - 1);
                            } else {
                                prev.bookmarks_count = prev.bookmarks_count.map(|v| v + 1);
                            }
                            prev.current_user_bookmarked = Some(
                                data.bookmark_blog_post
                                    .as_ref()
                                    .unwrap_or(&Default::default())
                                    .get_data(),
                            );
                        }
                    });
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn clear_current(&self) {
        self.current_blog_post.set(None);
    }

    /// Fetches featured blog posts and hydrates each post's author
    /// (`full_author_details`) before publishing.
    pub fn fetch_featured_posts(&self) {
        let featured_posts_sig = self.featured_posts;
        let loading = self.is_loading;
        let ui = self.ui;
        let user = self.user;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let vars = FetchBlogPostsVars {
                filters: FetchBlogPostsQueryFilters {
                    is_featured: Some(true),
                    status: Some(BlogStatus::Published),
                    ..Default::default()
                },
            };

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response =
                perform_mutation_or_query_with_vars::<FetchBlogPostsResponse, FetchBlogPostsVars>(
                    Some(&headers),
                    shared_service_api,
                    FEATURED_POSTS_QUERY,
                    vars,
                )
                .await;

            let Some(data) = response.get_data() else {
                handle_graphql_errors(&response, &ui, None);
                loading.set(false);
                return;
            };

            let mut posts = data
                .fetch_blog_posts
                .as_ref()
                .unwrap_or(&Default::default())
                .get_data()
                .to_vec();

            for post in posts.iter_mut() {
                if let Some(author_id) = post.author.clone() {
                    post.full_author_details = user.fetch_user_by_id_async(author_id).await;
                }
            }

            featured_posts_sig.set(posts);
            loading.set(false);
        });
    }

    /// Fetches non-featured blog posts. Does **not** hydrate authors — the
    /// `BlogPostPreview` card doesn't need them.
    pub fn fetch_other_posts(&self, filters: FetchBlogPostsQueryFilters) {
        let other_posts_sig = self.other_posts;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let vars = FetchBlogPostsVars { filters };

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                FetchBlogPostsResponse,
                FetchBlogPostsVars,
            >(
                Some(&headers), shared_service_api, OTHER_POSTS_QUERY, vars
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let posts = data
                        .fetch_blog_posts
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    other_posts_sig.set(posts);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Search-as-you-type. Writes into `search_results` and manages
    /// `search_is_loading` (separate from `is_loading` so the search overlay
    /// doesn't flicker when unrelated blog fetches run).
    pub fn search_blog_posts(&self, search_term: String) {
        let results_sig = self.search_results;
        let loading = self.search_is_loading;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let vars = FetchBlogPostsVars {
                filters: FetchBlogPostsQueryFilters {
                    search_term: Some(search_term),
                    ..Default::default()
                },
            };

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                FetchBlogPostsResponse,
                FetchBlogPostsVars,
            >(
                Some(&headers), shared_service_api, SEARCH_POSTS_QUERY, vars
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let posts = data
                        .fetch_blog_posts
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    results_sig.set(posts);
                }
                None => {
                    results_sig.set(vec![]);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `subscription_created_dirty`.
    pub fn create_subscription(&self, subscriber: SubscriberInput) {
        let dirty = self.subscription_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(mailing_list_id) = NEWSLETTER_MAILING_LIST_ID else {
                loading.set(false);
                return;
            };

            let vars = CreateSubscriptionVars {
                subscription_input: SubscriptionInput {
                    subscriber,
                    subscription_input_metadata: SubscriptionInputMetadata {
                        mailing_list_id: mailing_list_id.into(),
                    },
                },
            };

            let query = r#"
                    mutation SubscribeToMailingList($subscriptionInput: SubscriptionInput!) {
                        subscribeToMailingList(subscriptionInput: $subscriptionInput) {
                            data {
                                createdAt
                                id
                                mailingList { name description createdAt id }
                                subscriber {
                                    email firstName lastName status
                                    createdAt updatedAt id
                                }
                            }
                            metadata { requestId newAccessToken }
                        }
                    }
                "#;

            let Some(email_service_api) = EMAIL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateSubscriptionResponse,
                CreateSubscriptionVars,
            >(Some(&headers), email_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(_data) => {
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn clear_search(&self) {
        self.search_results.set(Vec::new());
    }
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_blog(ui: UiContext, user: UserContext, auth: AuthContext) -> BlogContext {
    let blog_ctx = BlogContext::new(ui, user, auth);
    provide_context(blog_ctx);
    blog_ctx
}

pub fn use_blog() -> BlogContext {
    expect_context::<BlogContext>()
}
