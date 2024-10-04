use floem::{
    keyboard::{Key, Modifiers, NamedKey},
    reactive::{RwSignal, SignalGet, SignalUpdate},
    style::Style,
    views::{button, container, dyn_container, h_stack, text_input, v_stack, Decorators},
    View,
};
use floem_async_img::async_image;

#[cfg(feature = "cache")]
use floem::reactive::provide_context;

#[cfg(feature = "cache")]
use floem_async_img::cache::{AsyncCache, CacheConfig};

#[cfg(all(
    not(feature = "async-std"),
    not(feature = "tokio"),
    not(feature = "smol"),
    not(feature = "thread"),
))]
fn main() {
    panic!("Enable at least one runtime with feature flag. Available runtimes/flags: async-std, tokio, smol, thread");
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
    floem::launch(create_view);
}

#[cfg(all(feature = "async-std", not(feature = "thread")))]
#[async_std::main]
async fn main() {
    floem::launch(|| create_view());
}

#[cfg(feature = "smol")]
fn main() {
    std::env::set_var("SMOL_THREADS", "4");

    smol::block_on(async {
        floem::launch(create_view);
    });
}

#[cfg(feature = "thread")]
fn main() {
    floem::launch(create_view);
}

#[allow(dead_code)]
fn create_view() -> impl View {
    let image_url = RwSignal::new(String::from("https://i.imgur.com/3Xi8Fg1.png"));
    let placeholder = include_bytes!("../assets/placeholder.png");

    let show = RwSignal::new(false);

    #[cfg(feature = "cache")]
    {
        let config = CacheConfig {
            placeholder: Some(include_bytes!("../assets/placeholder.png").to_vec().into()),
            ..Default::default()
        };

        let image_cache = AsyncCache::with_config(config);

        provide_context(image_cache);
    }

    let image = dyn_container(
        move || show.get(),
        move |_show| {
            async_image(image_url.get())
                .placeholder(placeholder.to_vec())
                .style(Style::size_full)
        },
    )
    .style(|s| s.size_full().justify_center().items_center());

    let input = text_input(image_url).style(|s| s.width_full());

    let button = button("Fetch")
        .style(|s| s.min_width(80))
        .action(move || show.update(|v| *v = !*v));

    let top_elements = h_stack((input, button));

    let view = container(v_stack((top_elements, image)).style(Style::size_full))
        .style(Style::size_full)
        .keyboard_navigatable();

    let id = view.id();

    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| {
        id.inspect()
    })
}
