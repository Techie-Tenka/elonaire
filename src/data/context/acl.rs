use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::graphql::acl::{
            CreateDepartmentResponse, CreateDepartmentVars, CreateOrganizationResponse,
            CreateOrganizationVars, CreatePermissionResponse, CreatePermissionVars,
            CreateResourceResponse, CreateResourceVars, CreateSystemRoleResponse,
            CreateSystemRoleVars, Department, DepartmentInput, DepartmentMetadata,
            FetchDepartmentsResponse, FetchOrganizationsResponse, FetchPermissionsResponse,
            FetchResourcesResponse, FetchSystemRolesResponse, Organization, OrganizationInput,
            Permission, PermissionInput, PermissionMetadata, Resource, ResourceInput,
            ResourceMetadata, RoleInput, RoleMetadata, SystemRole,
        },
    },
    utils::{
        errors::handle_graphql_errors,
        graphql_client::{perform_mutation_or_query_with_vars, perform_query_without_vars},
    },
};

const ACL_SERVICE_API: Option<&str> = option_env!("ACL_SERVICE_API");

#[derive(Clone, Debug, Copy)]
pub struct AclContext {
    ui: UiContext,
    auth: AuthContext,
    pub departments: RwSignal<Vec<Department>>,
    pub organizations: RwSignal<Vec<Organization>>,
    pub permissions: RwSignal<Vec<Permission>>,
    pub resources: RwSignal<Vec<Resource>>,
    pub roles: RwSignal<Vec<SystemRole>>,
    pub is_loading: RwSignal<bool>,
    pub department_created_dirty: RwSignal<u64>,
    pub organization_created_dirty: RwSignal<u64>,
    pub permission_created_dirty: RwSignal<u64>,
    pub resource_created_dirty: RwSignal<u64>,
    pub system_role_created_dirty: RwSignal<u64>,
}

impl AclContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            departments: RwSignal::new(Vec::new()),
            organizations: RwSignal::new(Vec::new()),
            permissions: RwSignal::new(Vec::new()),
            resources: RwSignal::new(Vec::new()),
            roles: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            department_created_dirty: RwSignal::new(0),
            organization_created_dirty: RwSignal::new(0),
            permission_created_dirty: RwSignal::new(0),
            resource_created_dirty: RwSignal::new(0),
            system_role_created_dirty: RwSignal::new(0),
        }
    }

    pub fn fetch_departments(&self) {
        let departments_sig = self.departments;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_departments_query = r#"
                   query FetchDepartments {
                        fetchDepartments {
                            data {
                                depName
                                createdAt
                                updatedAt
                                id
                                createdBy
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchDepartmentsResponse>(
                Some(&headers),
                acl_service_api,
                fetch_departments_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_departments
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    departments_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_organizations(&self) {
        let organizations_sig = self.organizations;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_orgs_query = r#"
                   query FetchOrganizations {
                        fetchOrganizations {
                            data {
                                orgName
                                createdAt
                                updatedAt
                                id
                                createdBy
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchOrganizationsResponse>(
                Some(&headers),
                acl_service_api,
                fetch_orgs_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_organizations
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    organizations_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_permissions(&self) {
        let permissions_sig = self.permissions;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_permissions_query = r#"
                   query FetchCurrentRolePermissions {
                        fetchCurrentRolePermissions {
                            data {
                                name
                                isAdmin
                                isSuperAdmin
                                id
                                createdBy
                                resource {
                                    name
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchPermissionsResponse>(
                Some(&headers),
                acl_service_api,
                fetch_permissions_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_current_role_permissions
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    permissions_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_resources(&self) {
        let resources_sig = self.resources;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_resources_query = r#"
                   query FetchResources {
                        fetchResources {
                            data {
                                name
                                id
                                createdBy
                                createdAt
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchResourcesResponse>(
                Some(&headers),
                acl_service_api,
                fetch_resources_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    resources_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_roles(&self) {
        let roles_sig = self.roles;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_roles_query = r#"
                   query FetchSystemRoles {
                        fetchSystemRoles {
                            data {
                                roleName
                                isAdmin
                                isDefault
                                isSuperAdmin
                                id
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSystemRolesResponse>(
                Some(&headers),
                acl_service_api,
                fetch_roles_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_system_roles
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    roles_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `department_created_dirty`.
    pub fn create_department(
        &self,
        department_input: DepartmentInput,
        department_metadata: DepartmentMetadata,
    ) {
        let dirty = self.department_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateDepartmentVars {
                department_input,
                department_metadata,
            };

            let query = r#"
                       mutation CreateDepartment($departmentInput: DepartmentInput!, $departmentMetadata: DepartmentMetadata!) {
                            createDepartment(departmentInput: $departmentInput, departmentMetadata: $departmentMetadata) {
                                data {
                                    depName
                                    createdAt
                                    updatedAt
                                    id
                                    createdBy
                                }
                                metadata {
                                    newAccessToken
                                    requestId
                                }
                            }
                       }
                   "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateDepartmentResponse,
                CreateDepartmentVars,
            >(Some(&headers), acl_service_api, query, input_vars)
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

    /// Fire-and-forget. Caller observes `organization_created_dirty`.
    pub fn create_organization(&self, organization_input: OrganizationInput) {
        let dirty = self.organization_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateOrganizationVars { organization_input };

            let query = r#"
                       mutation CreateOrganization($organizationInput: OrganizationInput!) {
                            createOrganization(organizationInput: $organizationInput) {
                                data {
                                    orgName
                                    createdAt
                                    updatedAt
                                    id
                                    createdBy
                                }
                                metadata {
                                    newAccessToken
                                    requestId
                                }
                            }
                       }
                   "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateOrganizationResponse,
                CreateOrganizationVars,
            >(Some(&headers), acl_service_api, query, input_vars)
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

    /// Fire-and-forget. Caller observes `permission_created_dirty`.
    pub fn create_permission(
        &self,
        permission_input: PermissionInput,
        permission_metadata: PermissionMetadata,
    ) {
        let dirty = self.permission_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreatePermissionVars {
                permission_input,
                permission_metadata,
            };

            let query = r#"
                   mutation CreatePermission($permissionInput: PermissionInput!, $permissionMetadata: PermissionMetadata!) {
                        createPermission(permissionInput: $permissionInput, permissionMetadata: $permissionMetadata) {
                            data {
                                name
                                isAdmin
                                isSuperAdmin
                                createdAt
                                updatedAt
                                id
                                createdBy
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreatePermissionResponse,
                CreatePermissionVars,
            >(Some(&headers), acl_service_api, query, input_vars)
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

    /// Fire-and-forget. Caller observes `resource_created_dirty`.
    pub fn create_resource(
        &self,
        resource_input: ResourceInput,
        resource_metadata: ResourceMetadata,
    ) {
        let dirty = self.resource_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateResourceVars {
                resource_input,
                resource_metadata,
            };

            let query = r#"
                   mutation CreateResource($resourceInput: ResourceInput!, $resourceMetadata: ResourceMetadata!) {
                        createResource(resourceInput: $resourceInput, resourceMetadata: $resourceMetadata) {
                            data {
                                name
                                id
                                createdBy
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateResourceResponse,
                CreateResourceVars,
            >(Some(&headers), acl_service_api, query, input_vars)
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

    /// Fire-and-forget. Caller observes `system_role_created_dirty`.
    pub fn create_system_role(&self, role_input: RoleInput, role_metadata: RoleMetadata) {
        let dirty = self.system_role_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateSystemRoleVars {
                role_input,
                role_metadata,
            };

            let query = r#"
                   mutation CreateSystemRole($roleInput: RoleInput!, $roleMetadata: RoleMetadata!) {
                        createSystemRole(roleInput: $roleInput, roleMetadata: $roleMetadata) {
                            data {
                                roleName
                                createdAt
                                isAdmin
                                isDefault
                                isSuperAdmin
                                updatedAt
                                id
                                createdBy
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateSystemRoleResponse,
                CreateSystemRoleVars,
            >(Some(&headers), acl_service_api, query, input_vars)
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
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_acl(ui: UiContext, auth: AuthContext) -> AclContext {
    let acl_ctx = AclContext::new(ui, auth);
    provide_context(acl_ctx);
    acl_ctx
}

pub fn use_acl() -> AclContext {
    expect_context::<AclContext>()
}
