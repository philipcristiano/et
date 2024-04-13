use maud::{html, Markup};

// heroicons https://heroicons.com/outline
fn default_hw() -> Markup {
    html! { "w-3 h-3 lg:w-6 lg:h-6" }
}

pub fn exclamation_circle() -> Markup {
    html! {
        svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class=(default_hw())
        {
            path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z"
                {}
        }
    }
}

pub fn x_circle() -> Markup {
    html! {
        svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class=(default_hw())
        {
            path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m9.75 9.75 4.5 4.5m0-4.5-4.5 4.5M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
                {}
        }
    }
}

pub fn magnifying_glass_minus() -> Markup {
    html! {
        svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class=(default_hw())
        {
            path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607ZM13.5 10.5h-6"

                {}
        }
    }
}

pub fn magnifying_glass_plus() -> Markup {
    html! {
        svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class=(default_hw())
        {
            path
               stroke-linecap="round"
               stroke-linejoin="round"
               d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607ZM10.5 7.5v6m3-3h-6"

                {}
        }
    }
}
