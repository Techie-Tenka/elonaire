use std::collections::HashMap;

use detaxine_ui::components::{
    data_display::table::data_table::{Column, DataTable, TableCellData},
    feedback::spinner::Spinner,
    navigation::breadcrumbs::Breadcrumbs,
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::Outlet;

use crate::data::context::billing::use_billing;

#[component]
pub fn ServiceRequests() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn ServiceRequestsList() -> impl IntoView {
    let billing_ctx = use_billing();
    let service_requests = move || billing_ctx.service_requests;

    let table_data = RwSignal::new((
        vec![
            Column::new("Description", false),
            Column::new("Start Date", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        billing_ctx.fetch_service_requests();
    });

    Effect::new(move || {
        let service_requests_rows: Vec<HashMap<String, TableCellData>> = service_requests()
            .get()
            .iter()
            .map(|service_request| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".into(),
                    TableCellData::String(
                        service_request
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Description".into(),
                    TableCellData::String(
                        service_request
                            .description
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Start Date".into(),
                    TableCellData::DateTime(
                        service_request
                            .start_date
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = service_requests_rows;
        });
    });

    view! {
        <>
            <Title text="Service Requests"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Service Requests"] />
            </div>
            <Show when=move || billing_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Service Requests</h1>

            <div class="display-constraints">
                <DataTable data=table_data editable=true deletable=true />
            </div>
        </>
    }
    .into_any()
}
