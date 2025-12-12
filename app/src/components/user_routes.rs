use crate::api::*;
use crate::components_impl::{
    build_user_route_form_dto, calculate_total_passengers, extract_user_route_form_state,
    PassengerCountData,
};
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
fn window() -> web_sys::Window {
    web_sys::window().expect("no global window exists")
}

#[component]
pub fn UserRoutesPage() -> impl IntoView {
    let users_resource = Resource::new(|| (), |()| async { get_users().await });
    let (selected_user_id, set_selected_user_id) = signal::<Option<String>>(None);
    let (show_form, set_show_form) = signal(false);
    let (editing_route, set_editing_route) = signal::<Option<UserRouteWithPassengersDto>>(None);

    let routes_resource = Resource::new(
        move || selected_user_id.get(),
        |user_id| async move {
            match user_id {
                Some(id) => get_user_routes(id).await,
                None => Ok(vec![]),
            }
        },
    );

    let refetch_routes = move || routes_resource.refetch();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-surface-900">"Routes"</h1>
                    <p class="mt-1 text-sm text-surface-500">"Configure bus routes and passenger details for each user"</p>
                </div>
                <button
                    class="btn-primary"
                    disabled=move || selected_user_id.get().is_none()
                    on:click=move |_| {
                        set_editing_route.set(None);
                        set_show_form.set(true);
                    }
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add Route"
                </button>
            </div>

            <Suspense fallback=move || view! { <UserSelectorSkeleton/> }>
                {move || {
                    users_resource
                        .get()
                        .map(|result| match result {
                            Ok(users) => {
                                view! {
                                    <UserSelector
                                        users=users
                                        on_select=move |id| set_selected_user_id.set(id)
                                    />
                                }
                                    .into_any()
                            }
                            Err(e) => {
                                view! { <p class="text-danger-600">"Error loading users: " {e.to_string()}</p> }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>

            <Show when=move || show_form.get()>
                <UserRouteFormModal
                    route=editing_route
                    user_id=selected_user_id.get().unwrap_or_default()
                    on_close=move || {
                        set_show_form.set(false);
                        set_editing_route.set(None);
                    }
                    on_save=move || {
                        set_show_form.set(false);
                        set_editing_route.set(None);
                        refetch_routes();
                    }
                />
            </Show>

            <Show when=move || selected_user_id.get().is_some()>
                <Suspense fallback=move || view! { <RoutesTableSkeleton/> }>
                    {move || {
                        routes_resource
                            .get()
                            .map(|result| match result {
                                Ok(routes) => {
                                    if routes.is_empty() {
                                        view! { <RoutesEmptyState on_add=move || set_show_form.set(true)/> }.into_any()
                                    } else {
                                        view! {
                                            <UserRoutesTable
                                                routes=routes
                                                on_edit=move |route: UserRouteWithPassengersDto| {
                                                    set_editing_route.set(Some(route));
                                                    set_show_form.set(true);
                                                }
                                                on_delete=move |id: String| {
                                                    leptos::task::spawn_local(async move {
                                                        if delete_user_route(id).await.is_ok() {
                                                            refetch_routes();
                                                        }
                                                    });
                                                }
                                            />
                                        }.into_any()
                                    }
                                }
                                Err(e) => {
                                    view! { <p class="text-danger-600">"Error: " {e.to_string()}</p> }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </Show>
        </div>
    }
}

#[component]
fn UserSelectorSkeleton() -> impl IntoView {
    view! {
        <div class="card p-4">
            <div class="skeleton h-4 w-24 mb-2"/>
            <div class="skeleton-input max-w-md"/>
        </div>
    }
}

#[component]
fn RoutesTableSkeleton() -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="table">
                <thead class="table-header">
                    <tr>
                        <th class="table-header-cell">"Route"</th>
                        <th class="table-header-cell">"Stations"</th>
                        <th class="table-header-cell">"Dates"</th>
                        <th class="table-header-cell">"Passengers"</th>
                        <th class="table-header-cell text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="table-body">
                    {(0..4).map(|_| view! {
                        <tr class="table-row">
                            <td class="table-cell"><div class="skeleton-text w-32"/></td>
                            <td class="table-cell"><div class="skeleton-text w-40"/></td>
                            <td class="table-cell"><div class="skeleton-text w-36"/></td>
                            <td class="table-cell"><div class="skeleton h-5 w-16 rounded-full"/></td>
                            <td class="table-cell">
                                <div class="flex justify-end gap-2">
                                    <div class="skeleton h-8 w-16 rounded-lg"/>
                                    <div class="skeleton h-8 w-16 rounded-lg"/>
                                </div>
                            </td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn RoutesEmptyState(on_add: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="card text-center py-12">
            <div class="w-12 h-12 bg-surface-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg class="w-6 h-6 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"/>
                </svg>
            </div>
            <h3 class="text-sm font-medium text-surface-900 mb-1">"No routes configured"</h3>
            <p class="text-sm text-surface-500 mb-4">"Get started by adding a route for this user"</p>
            <button
                class="btn-primary btn-sm"
                on:click=move |_| on_add()
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                </svg>
                "Add Route"
            </button>
        </div>
    }
}

#[component]
fn UserSelector(
    users: Vec<UserDto>,
    on_select: impl Fn(Option<String>) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="card p-4">
            <label class="form-label mb-2">"Select User"</label>
            <select
                class="form-select max-w-md"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if value.is_empty() {
                        on_select(None);
                    } else {
                        on_select(Some(value));
                    }
                }
            >
                <option value="">"-- Select a user --"</option>
                {users
                    .into_iter()
                    .map(|user| {
                        view! { <option value={user.id.clone()}>{user.email}</option> }
                    })
                    .collect_view()}
            </select>
        </div>
    }
}

#[component]
fn UserRoutesTable(
    routes: Vec<UserRouteWithPassengersDto>,
    on_edit: impl Fn(UserRouteWithPassengersDto) + 'static + Copy,
    on_delete: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="table">
                <thead class="table-header">
                    <tr>
                        <th class="table-header-cell">"Route"</th>
                        <th class="table-header-cell">"Stations"</th>
                        <th class="table-header-cell">"Dates"</th>
                        <th class="table-header-cell">"Passengers"</th>
                        <th class="table-header-cell text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="table-body">
                    {routes
                        .into_iter()
                        .map(|route| {
                            view! { <RouteRow route=route on_edit=on_edit on_delete=on_delete /> }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn RouteRow(
    route: UserRouteWithPassengersDto,
    on_edit: impl Fn(UserRouteWithPassengersDto) + 'static + Copy,
    on_delete: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    let route_clone = route.clone();
    let route_id = route.id.clone();
    let total_passengers = calculate_total_passengers(
        route.adult_men,
        route.adult_women,
        route.child_men,
        route.child_women,
        route.handicap_adult_men,
        route.handicap_adult_women,
        route.handicap_child_men,
        route.handicap_child_women,
    );

    view! {
        <tr class="table-row">
            <td class="table-cell">
                <div class="font-medium text-surface-900">"Area " {route.area_id}</div>
                <div class="text-sm text-surface-500">"Route " {route.route_id}</div>
            </td>
            <td class="table-cell">
                <div class="text-surface-900">{route.departure_station.clone()}</div>
                <div class="text-sm text-surface-500">"→ " {route.arrival_station.clone()}</div>
            </td>
            <td class="table-cell">
                <div class="text-surface-900">{route.date_start.clone()}</div>
                <div class="text-sm text-surface-500">"to " {route.date_end.clone()}</div>
            </td>
            <td class="table-cell">
                <span class="badge-info">{total_passengers} " passengers"</span>
            </td>
            <td class="table-cell">
                <div class="flex items-center justify-end gap-2">
                    <button
                        type="button"
                        class="btn-ghost btn-sm"
                        on:click=move |_| on_edit(route_clone.clone())
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                        </svg>
                        "Edit"
                    </button>
                    <button
                        type="button"
                        class="btn-ghost btn-sm text-danger-600 hover:text-danger-700 hover:bg-danger-50"
                        on:click={
                            let uid = route_id.clone();
                            move |_| {
                                #[cfg(target_arch = "wasm32")]
                                if window()
                                    .confirm_with_message("Are you sure you want to delete this route?")
                                    .unwrap_or(false)
                                {
                                    on_delete(uid.clone());
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                on_delete(uid.clone());
                            }
                        }
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                        </svg>
                        "Delete"
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn UserRouteFormModal(
    route: ReadSignal<Option<UserRouteWithPassengersDto>>,
    user_id: String,
    on_close: impl Fn() + 'static + Copy,
    on_save: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let is_edit = move || route.get().is_some();
    let initial = extract_user_route_form_state(route.get().as_ref());

    let (area_id, set_area_id) = signal(initial.area_id);
    let (route_id_val, set_route_id_val) = signal(initial.route_id);
    let (departure_station, set_departure_station) = signal(initial.departure_station);
    let (arrival_station, set_arrival_station) = signal(initial.arrival_station);
    let (date_start, set_date_start) = signal(initial.date_start);
    let (date_end, set_date_end) = signal(initial.date_end);
    let (time_min, set_time_min) = signal(initial.time_min);
    let (time_max, set_time_max) = signal(initial.time_max);

    let (adult_men, set_adult_men) = signal(initial.passengers.adult_men);
    let (adult_women, set_adult_women) = signal(initial.passengers.adult_women);
    let (child_men, set_child_men) = signal(initial.passengers.child_men);
    let (child_women, set_child_women) = signal(initial.passengers.child_women);
    let (handicap_adult_men, set_handicap_adult_men) = signal(initial.passengers.handicap_adult_men);
    let (handicap_adult_women, set_handicap_adult_women) =
        signal(initial.passengers.handicap_adult_women);
    let (handicap_child_men, set_handicap_child_men) = signal(initial.passengers.handicap_child_men);
    let (handicap_child_women, set_handicap_child_women) =
        signal(initial.passengers.handicap_child_women);

    let (is_saving, set_is_saving) = signal(false);

    let user_id_clone = user_id.clone();
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_saving.set(true);

        let passengers = PassengerCountData {
            adult_men: adult_men.get(),
            adult_women: adult_women.get(),
            child_men: child_men.get(),
            child_women: child_women.get(),
            handicap_adult_men: handicap_adult_men.get(),
            handicap_adult_women: handicap_adult_women.get(),
            handicap_child_men: handicap_child_men.get(),
            handicap_child_women: handicap_child_women.get(),
        };

        let form_data = build_user_route_form_dto(
            user_id_clone.clone(),
            area_id.get(),
            route_id_val.get(),
            departure_station.get(),
            arrival_station.get(),
            date_start.get(),
            date_end.get(),
            time_min.get(),
            time_max.get(),
            passengers,
        );

        let route_uuid = route.get().as_ref().map(|r| r.id.clone());
        let is_edit_mode = route_uuid.is_some();

        leptos::task::spawn_local(async move {
            let result = if is_edit_mode {
                update_user_route(route_uuid.unwrap_or_default(), form_data).await
            } else {
                create_user_route(form_data).await
            };

            if result.is_ok() {
                on_save();
            }
            set_is_saving.set(false);
        });
    };

    view! {
        <div class="modal-backdrop">
            <div class="modal-content-lg">
                <div class="modal-header flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-surface-900">
                        {move || if is_edit() { "Edit Route" } else { "Add Route" }}
                    </h2>
                    <button
                        type="button"
                        class="btn-ghost p-2 -mr-2 rounded-lg"
                        on:click=move |_| on_close()
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                    </button>
                </div>

                <form on:submit=handle_submit>
                    <div class="modal-body space-y-6">
                        <RouteSelectionSection
                            area_id=area_id
                            set_area_id=set_area_id
                            route_id=route_id_val
                            set_route_id=set_route_id_val
                            departure_station=departure_station
                            set_departure_station=set_departure_station
                            arrival_station=arrival_station
                            set_arrival_station=set_arrival_station
                            is_edit=is_edit
                        />

                        <DateTimeSection
                            date_start=date_start
                            set_date_start=set_date_start
                            date_end=date_end
                            set_date_end=set_date_end
                            time_min=time_min
                            set_time_min=set_time_min
                            time_max=time_max
                            set_time_max=set_time_max
                        />

                        <PassengersSection
                            adult_men=adult_men set_adult_men=set_adult_men
                            adult_women=adult_women set_adult_women=set_adult_women
                            child_men=child_men set_child_men=set_child_men
                            child_women=child_women set_child_women=set_child_women
                            handicap_adult_men=handicap_adult_men set_handicap_adult_men=set_handicap_adult_men
                            handicap_adult_women=handicap_adult_women set_handicap_adult_women=set_handicap_adult_women
                            handicap_child_men=handicap_child_men set_handicap_child_men=set_handicap_child_men
                            handicap_child_women=handicap_child_women set_handicap_child_women=set_handicap_child_women
                        />
                    </div>

                    <div class="modal-footer">
                        <button
                            type="button"
                            class="btn-secondary"
                            on:click=move |_| on_close()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="btn-primary"
                            disabled=move || is_saving.get()
                        >
                            {move || {
                                if is_saving.get() {
                                    view! {
                                        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
                                        </svg>
                                        "Saving..."
                                    }.into_any()
                                } else if is_edit() {
                                    view! { "Update" }.into_any()
                                } else {
                                    view! { "Create" }.into_any()
                                }
                            }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn RouteSelectionSection(
    area_id: ReadSignal<i32>,
    set_area_id: WriteSignal<i32>,
    route_id: ReadSignal<String>,
    set_route_id: WriteSignal<String>,
    departure_station: ReadSignal<String>,
    set_departure_station: WriteSignal<String>,
    arrival_station: ReadSignal<String>,
    set_arrival_station: WriteSignal<String>,
    is_edit: impl Fn() -> bool + 'static + Copy + Send,
) -> impl IntoView {
    // Routes depend on area_id
    let routes_for_area = Resource::new(
        move || area_id.get(),
        |area| async move { get_routes(area).await },
    );

    // Departure stations depend on route_id (fetched from API)
    let departure_stations = Resource::new(
        move || route_id.get(),
        |rid| async move {
            if rid.is_empty() {
                Ok(vec![])
            } else {
                get_departure_stations(rid).await
            }
        },
    );

    // Arrival stations depend on route_id AND departure_station (fetched from API)
    let arrival_stations = Resource::new(
        move || (route_id.get(), departure_station.get()),
        |(rid, dep)| async move {
            if rid.is_empty() || dep.is_empty() {
                Ok(vec![])
            } else {
                get_arrival_stations(rid, dep).await
            }
        },
    );

    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Route Selection"</legend>
            <div class="grid grid-cols-2 gap-4">
                <div class="form-group">
                    <label class="form-label">"Area"</label>
                    <select
                        class=move || if is_edit() { "form-select bg-surface-100 text-surface-500 cursor-not-allowed" } else { "form-select" }
                        disabled=is_edit
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                set_area_id.set(v);
                                set_route_id.set(String::new());
                                set_departure_station.set(String::new());
                                set_arrival_station.set(String::new());
                            }
                        }
                    >
                        <option value="1" selected=move || area_id.get() == 1>"Area 1"</option>
                        <option value="2" selected=move || area_id.get() == 2>"Area 2"</option>
                        <option value="3" selected=move || area_id.get() == 3>"Area 3"</option>
                    </select>
                </div>
                <div class="form-group">
                    <label class="form-label">"Route"</label>
                    {move || {
                        if is_edit() {
                            view! {
                                <input
                                    type="text"
                                    class="form-input bg-surface-100 text-surface-500 cursor-not-allowed"
                                    disabled=true
                                    value=move || format!("Route {}", route_id.get())
                                />
                            }.into_any()
                        } else {
                            view! {
                                <Suspense fallback=move || view! { <div class="skeleton-input"/> }>
                                    {move || routes_for_area.get().map(|result| {
                                        match result {
                                            Ok(routes) => view! {
                                                <RouteDropdown
                                                    routes=routes
                                                    selected=route_id
                                                    on_change=move |v| {
                                                        set_route_id.set(v);
                                                        set_departure_station.set(String::new());
                                                        set_arrival_station.set(String::new());
                                                    }
                                                />
                                            }.into_any(),
                                            Err(_) => view! { <select class="form-select" disabled><option>"Error loading routes"</option></select> }.into_any()
                                        }
                                    })}
                                </Suspense>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mt-4">
                <div class="form-group">
                    <label class="form-label form-label-required">"Departure Station"</label>
                    <Suspense fallback=move || view! { <div class="skeleton-input"/> }>
                        {move || departure_stations.get().map(|result| {
                            match result {
                                Ok(stations) => view! {
                                    <StationDropdown
                                        stations=stations
                                        selected=departure_station
                                        on_change=move |v| {
                                            set_departure_station.set(v);
                                            set_arrival_station.set(String::new());
                                        }
                                    />
                                }.into_any(),
                                Err(_) => view! { <select class="form-select" disabled><option>"Error loading stations"</option></select> }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
                <div class="form-group">
                    <label class="form-label form-label-required">"Arrival Station"</label>
                    <Suspense fallback=move || view! { <div class="skeleton-input"/> }>
                        {move || arrival_stations.get().map(|result| {
                            match result {
                                Ok(stations) => view! {
                                    <StationDropdown
                                        stations=stations
                                        selected=arrival_station
                                        on_change=move |v| set_arrival_station.set(v)
                                    />
                                }.into_any(),
                                Err(_) => view! { <select class="form-select" disabled><option>"Error loading stations"</option></select> }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </fieldset>
    }
}

#[component]
fn RouteDropdown(
    routes: Vec<RouteDto>,
    selected: ReadSignal<String>,
    on_change: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    view! {
        <select
            class="form-select"
            required
            prop:value=move || selected.get()
            on:change=move |ev| {
                on_change(event_target_value(&ev));
            }
        >
            <option value="">"-- Select route --"</option>
            {routes.into_iter().map(|r| {
                let rid = r.route_id.clone();
                let rid_display = rid.clone();
                view! {
                    <option value={rid}>
                        {r.name} " (" {rid_display} ")"
                    </option>
                }
            }).collect_view()}
        </select>
    }
}

#[component]
fn StationDropdown(
    stations: Vec<StationDto>,
    selected: ReadSignal<String>,
    on_change: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    view! {
        <select
            class="form-select"
            required
            on:change=move |ev| on_change(event_target_value(&ev))
        >
            <option value="" selected=move || selected.get().is_empty()>"-- Select station --"</option>
            {stations.into_iter().map(|s| {
                let sid = s.station_id.clone();
                let sid_check = sid.clone();
                view! {
                    <option
                        value={sid}
                        selected=move || selected.get() == sid_check
                    >
                        {s.name}
                    </option>
                }
            }).collect_view()}
        </select>
    }
}

#[component]
fn DateTimeSection(
    date_start: ReadSignal<String>,
    set_date_start: WriteSignal<String>,
    date_end: ReadSignal<String>,
    set_date_end: WriteSignal<String>,
    time_min: ReadSignal<String>,
    set_time_min: WriteSignal<String>,
    time_max: ReadSignal<String>,
    set_time_max: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Date & Time"</legend>
            <div class="grid grid-cols-2 gap-4">
                <div class="form-group">
                    <label class="form-label form-label-required">"Start Date"</label>
                    <input
                        type="date"
                        class="form-input"
                        required
                        prop:value=date_start
                        on:input=move |ev| set_date_start.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-group">
                    <label class="form-label form-label-required">"End Date"</label>
                    <input
                        type="date"
                        class="form-input"
                        required
                        prop:value=date_end
                        on:input=move |ev| set_date_end.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mt-4">
                <div class="form-group">
                    <label class="form-label">"Departure Time Min"</label>
                    <input
                        type="time"
                        class="form-input"
                        prop:value=time_min
                        on:input=move |ev| set_time_min.set(event_target_value(&ev))
                    />
                    <p class="form-hint">"Optional filter"</p>
                </div>
                <div class="form-group">
                    <label class="form-label">"Departure Time Max"</label>
                    <input
                        type="time"
                        class="form-input"
                        prop:value=time_max
                        on:input=move |ev| set_time_max.set(event_target_value(&ev))
                    />
                    <p class="form-hint">"Optional filter"</p>
                </div>
            </div>
        </fieldset>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn PassengersSection(
    adult_men: ReadSignal<i16>,
    set_adult_men: WriteSignal<i16>,
    adult_women: ReadSignal<i16>,
    set_adult_women: WriteSignal<i16>,
    child_men: ReadSignal<i16>,
    set_child_men: WriteSignal<i16>,
    child_women: ReadSignal<i16>,
    set_child_women: WriteSignal<i16>,
    handicap_adult_men: ReadSignal<i16>,
    set_handicap_adult_men: WriteSignal<i16>,
    handicap_adult_women: ReadSignal<i16>,
    set_handicap_adult_women: WriteSignal<i16>,
    handicap_child_men: ReadSignal<i16>,
    set_handicap_child_men: WriteSignal<i16>,
    handicap_child_women: ReadSignal<i16>,
    set_handicap_child_women: WriteSignal<i16>,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Passengers"</legend>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                <PassengerInput label="Adult Men" value=adult_men set_value=set_adult_men />
                <PassengerInput label="Adult Women" value=adult_women set_value=set_adult_women />
                <PassengerInput label="Child Men" value=child_men set_value=set_child_men />
                <PassengerInput label="Child Women" value=child_women set_value=set_child_women />
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mt-4">
                <PassengerInput label="Handicap Adult M" value=handicap_adult_men set_value=set_handicap_adult_men />
                <PassengerInput label="Handicap Adult W" value=handicap_adult_women set_value=set_handicap_adult_women />
                <PassengerInput label="Handicap Child M" value=handicap_child_men set_value=set_handicap_child_men />
                <PassengerInput label="Handicap Child W" value=handicap_child_women set_value=set_handicap_child_women />
            </div>
        </fieldset>
    }
}

#[component]
fn PassengerInput(
    label: &'static str,
    value: ReadSignal<i16>,
    set_value: WriteSignal<i16>,
) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label text-xs">{label}</label>
            <input
                type="number"
                min="0"
                class="form-input"
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse() {
                        set_value.set(v);
                    }
                }
            />
        </div>
    }
}
