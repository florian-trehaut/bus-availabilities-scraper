use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{A, Route, Router, Routes},
};

pub mod users;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Bus Scraper Dashboard"/>

        <Router>
            <nav class="bg-gray-800 text-white p-4">
                <div class="container mx-auto flex gap-4">
                    <A href="/" attr:class="hover:text-blue-400">"Home"</A>
                    <A href="/users" attr:class="hover:text-blue-400">"Users"</A>
                </div>
            </nav>

            <main class="container mx-auto p-4">
                <Routes fallback=|| "Page not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("users") view=users::UsersPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="text-center py-10">
            <h1 class="text-4xl font-bold mb-4">"Highway Bus Scraper Dashboard"</h1>
            <p class="text-gray-600 mb-8">"Monitor and manage your bus availability tracking"</p>
            <A href="/users" attr:class="bg-blue-600 text-white px-6 py-3 rounded hover:bg-blue-700">
                "Manage Users"
            </A>
        </div>
    }
}
