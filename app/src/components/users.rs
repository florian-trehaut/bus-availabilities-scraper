use crate::api::*;
use crate::components_impl::{
    build_user_form_dto, extract_user_form_state, notify_mode_badge_class, notify_mode_text,
    user_status_badge_class, user_status_text,
};
use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
fn window() -> web_sys::Window {
    web_sys::window().expect("no global window exists")
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let users_resource = Resource::new(|| (), |()| async { get_users().await });
    let (show_form, set_show_form) = signal(false);
    let (editing_user, set_editing_user) = signal::<Option<UserDto>>(None);

    let refetch = move || users_resource.refetch();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-surface-900">"Users"</h1>
                    <p class="mt-1 text-sm text-surface-500">"Manage user accounts and notification preferences"</p>
                </div>
                <button
                    class="btn-primary"
                    on:click=move |_| {
                        set_editing_user.set(None);
                        set_show_form.set(true);
                    }
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add User"
                </button>
            </div>

            <Show when=move || show_form.get()>
                <UserForm
                    user=editing_user
                    on_close=move || {
                        set_show_form.set(false);
                        set_editing_user.set(None);
                    }
                    on_save=move || {
                        set_show_form.set(false);
                        set_editing_user.set(None);
                        refetch();
                    }
                />
            </Show>

            <Suspense fallback=move || view! { <UsersTableSkeleton/> }>
                {move || {
                    users_resource
                        .get()
                        .map(|result| match result {
                            Ok(users) => {
                                if users.is_empty() {
                                    view! { <UsersEmptyState on_add=move || set_show_form.set(true)/> }.into_any()
                                } else {
                                    view! {
                                        <UsersTable
                                            users=users
                                            on_edit=move |user: UserDto| {
                                                set_editing_user.set(Some(user));
                                                set_show_form.set(true);
                                            }
                                            on_delete=move |id: String| {
                                                leptos::task::spawn_local(async move {
                                                    if delete_user(id).await.is_ok() {
                                                        refetch();
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
        </div>
    }
}

#[component]
fn UsersTableSkeleton() -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="table">
                <thead class="table-header">
                    <tr>
                        <th class="table-header-cell">"Email"</th>
                        <th class="table-header-cell">"Status"</th>
                        <th class="table-header-cell">"Interval"</th>
                        <th class="table-header-cell">"Notify"</th>
                        <th class="table-header-cell text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="table-body">
                    {(0..5).map(|_| view! {
                        <tr class="table-row">
                            <td class="table-cell"><div class="skeleton-text w-48"/></td>
                            <td class="table-cell"><div class="skeleton h-5 w-16 rounded-full"/></td>
                            <td class="table-cell"><div class="skeleton h-5 w-12 rounded-full"/></td>
                            <td class="table-cell"><div class="skeleton h-5 w-20 rounded-full"/></td>
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
fn UsersEmptyState(on_add: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="card text-center py-12">
            <div class="w-12 h-12 bg-surface-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg class="w-6 h-6 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
                </svg>
            </div>
            <h3 class="text-sm font-medium text-surface-900 mb-1">"No users yet"</h3>
            <p class="text-sm text-surface-500 mb-4">"Get started by adding your first user"</p>
            <button
                class="btn-primary btn-sm"
                on:click=move |_| on_add()
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                </svg>
                "Add User"
            </button>
        </div>
    }
}

#[component]
fn UsersTable(
    users: Vec<UserDto>,
    on_edit: impl Fn(UserDto) + 'static + Copy,
    on_delete: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="table">
                <thead class="table-header">
                    <tr>
                        <th class="table-header-cell">"Email"</th>
                        <th class="table-header-cell">"Status"</th>
                        <th class="table-header-cell">"Interval"</th>
                        <th class="table-header-cell">"Notify"</th>
                        <th class="table-header-cell text-right">"Actions"</th>
                    </tr>
                </thead>
                <tbody class="table-body">
                    {users
                        .into_iter()
                        .map(|user| {
                            let user_clone = user.clone();
                            let user_id = user.id.clone();
                            view! {
                                <tr class="table-row">
                                    <td class="table-cell font-medium text-surface-900">
                                        {user.email.clone()}
                                    </td>
                                    <td class="table-cell">
                                        <span class={user_status_badge_class(user.enabled)}>
                                            {user_status_text(user.enabled)}
                                        </span>
                                    </td>
                                    <td class="table-cell">
                                        <span class="badge-neutral">{user.scrape_interval_secs}"s"</span>
                                    </td>
                                    <td class="table-cell">
                                        <span class={notify_mode_badge_class(user.notify_on_change_only)}>
                                            {notify_mode_text(user.notify_on_change_only)}
                                        </span>
                                    </td>
                                    <td class="table-cell">
                                        <div class="flex items-center justify-end gap-2">
                                            <button
                                                type="button"
                                                class="btn-ghost btn-sm"
                                                on:click=move |_| on_edit(user_clone.clone())
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
                                                    let uid = user_id.clone();
                                                    move |_| {
                                                        #[cfg(target_arch = "wasm32")]
                                                        if window()
                                                            .confirm_with_message("Are you sure you want to delete this user?")
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
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn UserForm(
    user: ReadSignal<Option<UserDto>>,
    on_close: impl Fn() + 'static + Copy,
    on_save: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let is_edit = move || user.get().is_some();

    let initial = extract_user_form_state(user.get().as_ref());
    let (email, set_email) = signal(initial.email);
    let (enabled, set_enabled) = signal(initial.enabled);
    let (notify_on_change, set_notify_on_change) = signal(initial.notify_on_change_only);
    let (interval, set_interval) = signal(initial.interval);
    let (webhook, set_webhook) = signal(initial.webhook);
    let (is_saving, set_is_saving) = signal(false);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_saving.set(true);

        let form_data = build_user_form_dto(
            email.get(),
            enabled.get(),
            notify_on_change.get(),
            interval.get(),
            webhook.get(),
        );

        let user_id = user.get().as_ref().map(|u| u.id.clone());
        let is_edit = user_id.is_some();

        leptos::task::spawn_local(async move {
            let result = if is_edit {
                update_user(user_id.unwrap_or_default(), form_data).await
            } else {
                create_user(form_data).await
            };

            if result.is_ok() {
                on_save();
            }
            set_is_saving.set(false);
        });
    };

    view! {
        <div class="modal-backdrop">
            <div class="modal-content">
                <div class="modal-header flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-surface-900">
                        {move || if is_edit() { "Edit User" } else { "Add User" }}
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
                    <div class="modal-body space-y-4">
                        <div class="form-group">
                            <label class="form-label form-label-required">"Email"</label>
                            <input
                                type="email"
                                class="form-input"
                                placeholder="user@example.com"
                                required
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="flex gap-6">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="checkbox"
                                    class="form-checkbox"
                                    prop:checked=enabled
                                    on:change=move |ev| set_enabled.set(event_target_checked(&ev))
                                />
                                <span class="text-sm text-surface-700">"Enabled"</span>
                            </label>

                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="checkbox"
                                    class="form-checkbox"
                                    prop:checked=notify_on_change
                                    on:change=move |ev| set_notify_on_change.set(event_target_checked(&ev))
                                />
                                <span class="text-sm text-surface-700">"Notify on Change Only"</span>
                            </label>
                        </div>

                        <div class="form-group">
                            <label class="form-label form-label-required">"Scrape Interval"</label>
                            <div class="relative">
                                <input
                                    type="number"
                                    class="form-input pr-16"
                                    required
                                    min="60"
                                    max="3600"
                                    prop:value=interval
                                    on:input=move |ev| set_interval.set(event_target_value(&ev))
                                />
                                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-surface-400">
                                    "seconds"
                                </span>
                            </div>
                            <p class="form-hint">"Min: 60s, Max: 3600s"</p>
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Discord Webhook URL"</label>
                            <input
                                type="url"
                                class="form-input"
                                placeholder="https://discord.com/api/webhooks/..."
                                prop:value=webhook
                                on:input=move |ev| set_webhook.set(event_target_value(&ev))
                            />
                            <p class="form-hint">"Optional - Leave empty to disable notifications"</p>
                        </div>
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
