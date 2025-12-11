use crate::api::*;
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
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold">"Users Management"</h1>
                <button
                    class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
                    on:click=move |_| {
                        set_editing_user.set(None);
                        set_show_form.set(true);
                    }
                >
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

            <Suspense fallback=move || view! { <p>"Loading users..."</p> }>
                {move || {
                    users_resource
                        .get()
                        .map(|result| match result {
                            Ok(users) => {
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
                                }
                                    .into_any()
                            }
                            Err(e) => {
                                view! { <p class="text-red-600">"Error: " {e.to_string()}</p> }
                                    .into_any()
                            }
                        })
                }}

            </Suspense>
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
        <div class="overflow-x-auto">
            <table class="min-w-full bg-white border border-gray-300">
                <thead class="bg-gray-100">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase">
                            "Email"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase">
                            "Status"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase">
                            "Interval (sec)"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase">
                            "Notify on Change"
                        </th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase">
                            "Actions"
                        </th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-200">
                    {users
                        .into_iter()
                        .map(|user| {
                            let user_clone = user.clone();
                            let user_id = user.id.clone();
                            view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-6 py-4 whitespace-nowrap">{user.email.clone()}</td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <span class={
                                            if user.enabled {
                                                "bg-green-100 text-green-800 px-2 py-1 rounded text-sm"
                                            } else {
                                                "bg-red-100 text-red-800 px-2 py-1 rounded text-sm"
                                            }
                                        }>{if user.enabled { "Active" } else { "Inactive" }}</span>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        {user.scrape_interval_secs}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        {if user.notify_on_change_only { "Yes" } else { "No" }}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap space-x-2">
                                        <button
                                            type="button"
                                            class="bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 text-sm"
                                            on:click=move |_| on_edit(user_clone.clone())
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            type="button"
                                            class="bg-red-500 text-white px-3 py-1 rounded hover:bg-red-600 text-sm"
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
                                            "Delete"
                                        </button>
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

    let (email, set_email) = signal(
        user.get()
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_default(),
    );
    let (enabled, set_enabled) = signal(user.get().as_ref().is_none_or(|u| u.enabled));
    let (notify_on_change, set_notify_on_change) =
        signal(user.get().as_ref().is_none_or(|u| u.notify_on_change_only));
    let (interval, set_interval) = signal(
        user.get()
            .as_ref()
            .map_or_else(|| "300".to_string(), |u| u.scrape_interval_secs.to_string()),
    );
    let (webhook, set_webhook) = signal(
        user.get()
            .as_ref()
            .and_then(|u| u.discord_webhook_url.clone())
            .unwrap_or_default(),
    );
    let (is_saving, set_is_saving) = signal(false);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_saving.set(true);

        let form_data = UserFormDto {
            email: email.get(),
            enabled: enabled.get(),
            notify_on_change_only: notify_on_change.get(),
            scrape_interval_secs: interval.get().parse().unwrap_or(300),
            discord_webhook_url: {
                let w = webhook.get();
                if w.is_empty() { None } else { Some(w) }
            },
        };

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
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg p-6 w-full max-w-md">
                <h2 class="text-2xl font-bold mb-4">
                    {move || if is_edit() { "Edit User" } else { "Add User" }}
                </h2>

                <form on:submit=handle_submit class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Email"
                        </label>
                        <input
                            type="email"
                            class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="flex items-center">
                        <input
                            type="checkbox"
                            id="enabled"
                            class="mr-2"
                            prop:checked=enabled
                            on:change=move |ev| set_enabled.set(event_target_checked(&ev))
                        />
                        <label for="enabled" class="text-sm font-medium text-gray-700">
                            "Enabled"
                        </label>
                    </div>

                    <div class="flex items-center">
                        <input
                            type="checkbox"
                            id="notify_on_change"
                            class="mr-2"
                            prop:checked=notify_on_change
                            on:change=move |ev| set_notify_on_change.set(event_target_checked(&ev))
                        />
                        <label for="notify_on_change" class="text-sm font-medium text-gray-700">
                            "Notify on Change Only"
                        </label>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Scrape Interval (seconds)"
                        </label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                            min="60"
                            max="3600"
                            prop:value=interval
                            on:input=move |ev| set_interval.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Discord Webhook URL (optional)"
                        </label>
                        <input
                            type="url"
                            class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=webhook
                            on:input=move |ev| set_webhook.set(event_target_value(&ev))
                        />
                    </div>

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
                                if is_saving.get() {
                                    "Saving..."
                                } else if is_edit() {
                                    "Update"
                                } else {
                                    "Create"
                                }
                            }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
