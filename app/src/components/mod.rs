use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{A, Route, Router, Routes},
};

pub mod user_routes;
pub mod users;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Bus Scraper Dashboard"/>

        <Router>
            <nav class="bg-surface-900 border-b border-surface-800 sticky top-0 z-40">
                <div class="container mx-auto">
                    <div class="flex items-center justify-between h-16">
                        <div class="flex items-center gap-2">
                            <div class="w-8 h-8 bg-primary-500 rounded-lg flex items-center justify-center">
                                <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                                </svg>
                            </div>
                            <span class="text-white font-semibold text-lg">"Bus Scraper"</span>
                        </div>

                        <div class="flex items-center gap-1">
                            <A href="/" attr:class="nav-link">
                                <svg class="w-4 h-4 inline-block mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
                                </svg>
                                "Home"
                            </A>
                            <A href="/users" attr:class="nav-link">
                                <svg class="w-4 h-4 inline-block mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
                                </svg>
                                "Users"
                            </A>
                            <A href="/user-routes" attr:class="nav-link">
                                <svg class="w-4 h-4 inline-block mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                          d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"/>
                                </svg>
                                "Routes"
                            </A>
                        </div>
                    </div>
                </div>
            </nav>

            <main class="container mx-auto p-4">
                <Routes fallback=|| "Page not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("users") view=users::UsersPage/>
                    <Route path=StaticSegment("user-routes") view=user_routes::UserRoutesPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-[60vh] text-center">
            <div class="w-16 h-16 bg-primary-100 rounded-2xl flex items-center justify-center mb-6">
                <svg class="w-8 h-8 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                </svg>
            </div>
            <h1 class="text-3xl font-bold text-surface-900 mb-3">
                "Highway Bus Scraper Dashboard"
            </h1>
            <p class="text-surface-600 mb-8 max-w-md">
                "Monitor and manage your bus availability tracking with real-time notifications"
            </p>
            <div class="flex gap-4">
                <A href="/users" attr:class="btn-primary">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"/>
                    </svg>
                    "Manage Users"
                </A>
                <A href="/user-routes" attr:class="btn-secondary">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"/>
                    </svg>
                    "Configure Routes"
                </A>
            </div>
        </div>
    }
}
