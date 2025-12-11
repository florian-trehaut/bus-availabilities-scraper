use crate::api::*;
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
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"User Routes Management"</h1>
                <button
                    class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
                    disabled=move || selected_user_id.get().is_none()
                    on:click=move |_| {
                        set_editing_route.set(None);
                        set_show_form.set(true);
                    }
                >
                    "Add Route"
                </button>
            </div>

            <Suspense fallback=move || view! { <p>"Loading users..."</p> }>
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
                                view! { <p class="text-red-600">"Error loading users: " {e.to_string()}</p> }
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
                <Suspense fallback=move || view! { <p>"Loading routes..."</p> }>
                    {move || {
                        routes_resource
                            .get()
                            .map(|result| match result {
                                Ok(routes) => {
                                    if routes.is_empty() {
                                        view! {
                                            <p class="text-gray-500 italic">"No routes configured for this user."</p>
                                        }
                                            .into_any()
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
                                        }
                                            .into_any()
                                    }
                                }
                                Err(e) => {
                                    view! { <p class="text-red-600">"Error: " {e.to_string()}</p> }
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
fn UserSelector(
    users: Vec<UserDto>,
    on_select: impl Fn(Option<String>) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1">"Select User"</label>
            <select
                class="w-full max-w-md px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
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
        <div class="overflow-x-auto">
            <table class="min-w-full bg-white border border-gray-300">
                <thead class="bg-gray-100">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase">"Route"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase">"Stations"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase">"Dates"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase">"Passengers"</th>
                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-700 uppercase">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200">
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
    let total_passengers = route.adult_men
        + route.adult_women
        + route.child_men
        + route.child_women
        + route.handicap_adult_men
        + route.handicap_adult_women
        + route.handicap_child_men
        + route.handicap_child_women;

    view! {
        <tr class="hover:bg-gray-50">
            <td class="px-4 py-4 whitespace-nowrap">
                <div class="text-sm font-medium">"Area " {route.area_id} " / Route " {route.route_id}</div>
            </td>
            <td class="px-4 py-4 whitespace-nowrap">
                <div class="text-sm">{route.departure_station.clone()} " → " {route.arrival_station.clone()}</div>
            </td>
            <td class="px-4 py-4 whitespace-nowrap">
                <div class="text-sm">{route.date_start.clone()} " to " {route.date_end.clone()}</div>
            </td>
            <td class="px-4 py-4 whitespace-nowrap">
                <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-sm">{total_passengers} " pax"</span>
            </td>
            <td class="px-4 py-4 whitespace-nowrap space-x-2">
                <button
                    type="button"
                    class="bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 text-sm"
                    on:click=move |_| on_edit(route_clone.clone())
                >
                    "Edit"
                </button>
                <button
                    type="button"
                    class="bg-red-500 text-white px-3 py-1 rounded hover:bg-red-600 text-sm"
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
                    "Delete"
                </button>
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
    let initial = route.get();

    let (area_id, set_area_id) = signal(initial.as_ref().map_or(1, |r| r.area_id));
    let (route_id_val, set_route_id_val) = signal(initial.as_ref().map_or(0, |r| r.route_id));
    let (departure_station, set_departure_station) = signal(
        initial
            .as_ref()
            .map_or_else(String::new, |r| r.departure_station.clone()),
    );
    let (arrival_station, set_arrival_station) = signal(
        initial
            .as_ref()
            .map_or_else(String::new, |r| r.arrival_station.clone()),
    );
    let (date_start, set_date_start) = signal(
        initial
            .as_ref()
            .map_or_else(String::new, |r| r.date_start.clone()),
    );
    let (date_end, set_date_end) = signal(
        initial
            .as_ref()
            .map_or_else(String::new, |r| r.date_end.clone()),
    );
    let (time_min, set_time_min) = signal(
        initial
            .as_ref()
            .and_then(|r| r.departure_time_min.clone())
            .unwrap_or_default(),
    );
    let (time_max, set_time_max) = signal(
        initial
            .as_ref()
            .and_then(|r| r.departure_time_max.clone())
            .unwrap_or_default(),
    );

    let (adult_men, set_adult_men) = signal(initial.as_ref().map_or(0i16, |r| r.adult_men));
    let (adult_women, set_adult_women) = signal(initial.as_ref().map_or(0i16, |r| r.adult_women));
    let (child_men, set_child_men) = signal(initial.as_ref().map_or(0i16, |r| r.child_men));
    let (child_women, set_child_women) = signal(initial.as_ref().map_or(0i16, |r| r.child_women));
    let (handicap_adult_men, set_handicap_adult_men) =
        signal(initial.as_ref().map_or(0i16, |r| r.handicap_adult_men));
    let (handicap_adult_women, set_handicap_adult_women) =
        signal(initial.as_ref().map_or(0i16, |r| r.handicap_adult_women));
    let (handicap_child_men, set_handicap_child_men) =
        signal(initial.as_ref().map_or(0i16, |r| r.handicap_child_men));
    let (handicap_child_women, set_handicap_child_women) =
        signal(initial.as_ref().map_or(0i16, |r| r.handicap_child_women));

    let (is_saving, set_is_saving) = signal(false);

    let user_id_clone = user_id.clone();
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_saving.set(true);

        let form_data = UserRouteFormDto {
            user_id: user_id_clone.clone(),
            area_id: area_id.get(),
            route_id: route_id_val.get(),
            departure_station: departure_station.get(),
            arrival_station: arrival_station.get(),
            date_start: date_start.get(),
            date_end: date_end.get(),
            departure_time_min: {
                let t = time_min.get();
                if t.is_empty() { None } else { Some(t) }
            },
            departure_time_max: {
                let t = time_max.get();
                if t.is_empty() { None } else { Some(t) }
            },
            adult_men: adult_men.get(),
            adult_women: adult_women.get(),
            child_men: child_men.get(),
            child_women: child_women.get(),
            handicap_adult_men: handicap_adult_men.get(),
            handicap_adult_women: handicap_adult_women.get(),
            handicap_child_men: handicap_child_men.get(),
            handicap_child_women: handicap_child_women.get(),
        };

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
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 overflow-y-auto py-4">
            <div class="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
                <h2 class="text-2xl font-bold mb-4">
                    {move || if is_edit() { "Edit Route" } else { "Add Route" }}
                </h2>

                <form on:submit=handle_submit class="space-y-6">
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

                    <div class="flex justify-end gap-2 pt-4">
                        <button
                            type="button"
                            class="px-4 py-2 border border-gray-300 rounded hover:bg-gray-100"
                            on:click=move |_| on_close()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                            disabled=move || is_saving.get()
                        >
                            {move || {
                                if is_saving.get() { "Saving..." }
                                else if is_edit() { "Update" }
                                else { "Create" }
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
    route_id: ReadSignal<i32>,
    set_route_id: WriteSignal<i32>,
    departure_station: ReadSignal<String>,
    set_departure_station: WriteSignal<String>,
    arrival_station: ReadSignal<String>,
    set_arrival_station: WriteSignal<String>,
    is_edit: impl Fn() -> bool + 'static + Copy + Send,
) -> impl IntoView {
    let routes_for_area = Resource::new(
        move || area_id.get(),
        |area| async move { get_routes(area).await },
    );

    let stations_for_route = Resource::new(
        move || (route_id.get(), area_id.get()),
        |(rid, aid)| async move {
            if rid == 0 {
                Ok(vec![])
            } else {
                get_stations_for_route(rid, aid).await
            }
        },
    );

    let area_class = move || {
        if is_edit() {
            "w-full px-3 py-2 border border-gray-300 rounded bg-gray-100 text-gray-500 cursor-not-allowed"
        } else {
            "w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        }
    };

    view! {
        <fieldset class="border border-gray-200 rounded p-4">
            <legend class="text-sm font-semibold text-gray-700 px-2">"Route Selection"</legend>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Area"</label>
                    <select
                        class=area_class
                        disabled=move || is_edit()
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                set_area_id.set(v);
                                set_route_id.set(0);
                            }
                        }
                    >
                        <option value="1" selected=move || area_id.get() == 1>"Area 1"</option>
                        <option value="2" selected=move || area_id.get() == 2>"Area 2"</option>
                        <option value="3" selected=move || area_id.get() == 3>"Area 3"</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Route"</label>
                    {move || {
                        if is_edit() {
                            view! {
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded bg-gray-100 text-gray-500 cursor-not-allowed"
                                    disabled=true
                                    value=format!("Route {}", route_id.get())
                                />
                            }.into_any()
                        } else {
                            view! {
                                <Suspense fallback=move || view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Loading..."</option></select> }>
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
                                            Err(_) => view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Error"</option></select> }.into_any()
                                        }
                                    })}
                                </Suspense>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mt-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Departure Station"</label>
                    <Suspense fallback=move || view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Loading..."</option></select> }>
                        {move || stations_for_route.get().map(|result| {
                            match result {
                                Ok(stations) => view! {
                                    <StationDropdown
                                        stations=stations
                                        selected=departure_station
                                        on_change=move |v| set_departure_station.set(v)
                                    />
                                }.into_any(),
                                Err(_) => view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Error"</option></select> }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Arrival Station"</label>
                    <Suspense fallback=move || view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Loading..."</option></select> }>
                        {move || stations_for_route.get().map(|result| {
                            match result {
                                Ok(stations) => view! {
                                    <StationDropdown
                                        stations=stations
                                        selected=arrival_station
                                        on_change=move |v| set_arrival_station.set(v)
                                    />
                                }.into_any(),
                                Err(_) => view! { <select class="w-full px-3 py-2 border rounded" disabled><option>"Error"</option></select> }.into_any()
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
    selected: ReadSignal<i32>,
    on_change: impl Fn(i32) + 'static + Copy,
) -> impl IntoView {
    view! {
        <select
            class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
            prop:value=move || selected.get().to_string()
            on:change=move |ev| {
                if let Ok(v) = event_target_value(&ev).parse() {
                    on_change(v);
                }
            }
        >
            <option value="0">"-- Select route --"</option>
            {routes.into_iter().map(|r| {
                let rid: i32 = r.route_id.parse().unwrap_or(0);
                let rid_str = rid.to_string();
                view! {
                    <option value={rid_str}>
                        {r.name} " (" {r.route_id} ")"
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
            class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
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
        <fieldset class="border border-gray-200 rounded p-4">
            <legend class="text-sm font-semibold text-gray-700 px-2">"Date & Time"</legend>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                    <input
                        type="date"
                        class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                        prop:value=date_start
                        on:input=move |ev| set_date_start.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                    <input
                        type="date"
                        class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                        prop:value=date_end
                        on:input=move |ev| set_date_end.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mt-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Departure Time Min (optional)"</label>
                    <input
                        type="time"
                        class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        prop:value=time_min
                        on:input=move |ev| set_time_min.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Departure Time Max (optional)"</label>
                    <input
                        type="time"
                        class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        prop:value=time_max
                        on:input=move |ev| set_time_max.set(event_target_value(&ev))
                    />
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
        <fieldset class="border border-gray-200 rounded p-4">
            <legend class="text-sm font-semibold text-gray-700 px-2">"Passengers"</legend>
            <div class="grid grid-cols-4 gap-4">
                <PassengerInput label="Adult Men" value=adult_men set_value=set_adult_men />
                <PassengerInput label="Adult Women" value=adult_women set_value=set_adult_women />
                <PassengerInput label="Child Men" value=child_men set_value=set_child_men />
                <PassengerInput label="Child Women" value=child_women set_value=set_child_women />
            </div>
            <div class="grid grid-cols-4 gap-4 mt-4">
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
        <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{label}</label>
            <input
                type="number"
                min="0"
                class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
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
