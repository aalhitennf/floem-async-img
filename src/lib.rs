#![allow(unused)]

#[cfg(feature = "cache")]
pub mod cache;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use floem::{
    ext_event::create_signal_from_channel,
    reactive::{create_effect, use_context, with_scope, RwSignal, Scope},
    reactive::{SignalGet, SignalUpdate},
    views::{img, Decorators, DynamicView, Img},
    IntoView, View,
};

#[cfg(feature = "cache")]
use self::cache::AsyncCache;

pub struct AsyncImage {
    cx: Scope,

    url: String,
    buffer: Bytes,
    fetch_channel: (Sender<Bytes>, Receiver<Bytes>),
}

impl AsyncImage {
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        let cx = Scope::new();

        let url: String = url.into();
        let fetch_channel = crossbeam_channel::bounded(1);

        Self {
            cx,

            url,
            buffer: Bytes::default(),
            fetch_channel,
        }
    }

    #[must_use]
    pub fn placeholder(mut self, bytes: impl Into<Bytes>) -> Self {
        self.buffer = bytes.into();
        self
    }

    // pub fn build(self) -> impl IntoView {
    //     let cx = self.cx;
    //     let url = self.url;

    //     let buffer = cx.create_rw_signal(self.buffer);

    //     let tx = cx.create_rw_signal(self.fetch_channel.0);
    //     let rx = self.fetch_channel.1;

    //     with_scope(cx, || async_image_view(url, buffer, tx, rx))
    // }
}

#[cfg(not(feature = "cache"))]
impl IntoView for AsyncImage {
    type V = Img;

    fn into_view(self) -> Self::V {
        let cx = self.cx;
        let url = self.url;

        let buffer = cx.create_rw_signal(self.buffer);

        let tx = cx.create_rw_signal(self.fetch_channel.0);
        let rx = self.fetch_channel.1;

        with_scope(cx, || async_image_view(url, buffer, tx, rx))
    }
}

#[cfg(feature = "cache")]
impl IntoView for AsyncImage {
    type V = Img;

    fn into_view(self) -> Self::V {
        let cx = self.cx;
        let url = self.url;

        let buffer = cx.create_rw_signal(self.buffer);

        let tx = cx.create_rw_signal(self.fetch_channel.0);
        let rx = self.fetch_channel.1;

        with_scope(cx, || async_image_view_cache(url, buffer, tx, rx))
    }
}

#[cfg(not(feature = "cache"))]
fn async_image_view(
    url: String,
    buffer: RwSignal<Bytes>,
    tx: RwSignal<Sender<Bytes>>,
    rx: Receiver<Bytes>,
) -> Img {
    use floem::views::{dyn_view, DynamicView};

    let image_signal = create_signal_from_channel(rx);

    create_effect(move |_| {
        if let Some(v) = image_signal.get() {
            buffer.set(v);
        }
    });

    create_effect(move |_| {
        fetch(url.clone(), tx.get_untracked());
    });

    img(move || buffer.get().to_vec())
}

pub fn async_image(url: impl Into<String>) -> AsyncImage {
    AsyncImage::new(url)
}

#[cfg(feature = "cache")]
fn async_image_view_cache(
    url: String,
    buffer: RwSignal<Bytes>,
    tx: RwSignal<Sender<Bytes>>,
    rx: Receiver<Bytes>,
) -> Img {
    let cache = use_context::<AsyncCache>().unwrap();

    let image_signal = create_signal_from_channel(rx);

    let image_url = RwSignal::new(url);

    create_effect(move |_| {
        if let Some(v) = image_signal.get() {
            buffer.set(v);
        }
    });

    create_effect(move |_| {
        cache.url(&tx.get_untracked(), &image_url.get_untracked());
    });

    img(move || buffer.get().to_vec())
}

#[inline]
fn fetch(url: String, sender: Sender<Bytes>) {
    #[cfg(feature = "tokio")]
    fetch_tokio(url.clone(), sender.clone());

    #[cfg(feature = "async-std")]
    fetch_async_std(url.clone(), sender.clone());

    #[cfg(feature = "smol")]
    fetch_async_smol(url.clone(), sender.clone());

    #[cfg(feature = "thread")]
    fetch_thread(url.clone(), sender.clone());
}

#[cfg(feature = "tokio")]
fn fetch_tokio(url: String, sender: Sender<Bytes>) {
    tokio::spawn(async move {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;

        if let Err(e) = sender.send(bytes) {
            eprintln!("{e}");
        }

        reqwest::Result::Ok(())
    });
}

#[cfg(feature = "async-std")]
fn fetch_async_std(url: String, sender: Sender<Bytes>) {
    async_std::task::spawn(async move {
        if let Err(e) = fetch_compat(url, sender).await {
            eprintln!("{e}");
        }
    });
}

#[cfg(feature = "smol")]
fn fetch_async_smol(url: String, sender: Sender<Bytes>) {
    smol::spawn(async move {
        if let Err(e) = fetch_compat(url, sender).await {
            eprintln!("{e}");
        }
    })
    .detach();
}

#[cfg(feature = "thread")]
fn fetch_thread(url: String, sender: Sender<Bytes>) {
    let _ = std::thread::spawn(move || {
        let response = reqwest::blocking::get(url)?;
        let bytes = response.bytes()?;
        if let Err(e) = sender.send(bytes) {
            eprintln!("{e}");
        }

        std::result::Result::<(), reqwest::Error>::Ok(())
    });
}

#[cfg(any(feature = "smol", feature = "async-std"))]
async fn fetch_compat(url: String, sender: Sender<Bytes>) -> Result<(), reqwest::Error> {
    use async_compat::CompatExt;

    let response = reqwest::get(url).compat().await?;
    let bytes = response.bytes().compat().await?;
    if let Err(e) = sender.send(bytes) {
        eprintln!("{e}");
    }

    Ok(())
}
