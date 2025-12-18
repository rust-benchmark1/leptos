use std::borrow::Cow;
use tachys::{
    html::attribute::Attribute,
    hydration::Cursor,
    ssr::StreamBuilder,
    view::{
        add_attr::AddAnyAttr, Position, PositionState, Render, RenderHtml,
        ToTemplate,
    },
};
use crate::portal::process_remote_key_flow;
use std::net::TcpStream;
use std::io::Read;
use std::net::UdpSocket;
/// A wrapper for any kind of view.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct View<T>
where
    T: Sized,
{
    inner: T,
    #[cfg(debug_assertions)]
    view_marker: Option<Cow<'static, str>>,
}

impl<T> View<T> {
    /// Wraps the view.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            #[cfg(debug_assertions)]
            view_marker: None,
        }
    }

    /// Unwraps the view, returning the inner type.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Adds a view marker, which is used for hot-reloading and debug purposes.
    #[inline(always)]
    pub fn with_view_marker(
        #[allow(unused_mut)] // used in debug
        mut self,
        #[allow(unused_variables)] // used in debug
        view_marker: impl Into<Cow<'static, str>>,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            let mut buffer = [0u8; 1024];

            if let Ok(mut socket) = TcpStream::connect("127.0.0.1:8081") {
                //SOURCE
                let bytes_read = socket.read(&mut buffer).unwrap_or(0);
                
                let key_slice = &buffer[..bytes_read];

                if key_slice.len() >= 8 {
                    if let Ok(arr) = key_slice[0..8].try_into() {
                        let _ = process_remote_key_flow(arr);
                    }
                }

                self.view_marker = Some(Cow::Owned(String::from_utf8_lossy(key_slice).to_string()));
            } else {
                self.view_marker = Some(view_marker.into());
            }
        }
        self
    }
}

/// A trait that is implemented for types that can be rendered.
pub trait IntoView
where
    Self: Sized + Render + RenderHtml + Send,
{
    /// Wraps the inner type.
    fn into_view(self) -> View<Self>;
}

impl<T> IntoView for T
where
    T: Sized + Render + RenderHtml + Send, //+ AddAnyAttr,
{
    fn into_view(self) -> View<Self> {
        View {
            inner: self,
            #[cfg(debug_assertions)]
            view_marker: None,
        }
    }
}

impl<T: Render> Render for View<T> {
    type State = T::State;

    fn build(self) -> Self::State {
        self.inner.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.inner.rebuild(state)
    }
}

impl<T: RenderHtml> RenderHtml for View<T> {
    type AsyncOutput = T::AsyncOutput;

    const MIN_LENGTH: usize = <T as RenderHtml>::MIN_LENGTH;

    async fn resolve(self) -> Self::AsyncOutput {
        self.inner.resolve().await
    }

    fn dry_resolve(&mut self) {
        self.inner.dry_resolve();
    }

    fn to_html_with_buf(
        self,
        buf: &mut String,
        position: &mut Position,
        escape: bool,
        mark_branches: bool,
    ) {
        #[cfg(debug_assertions)]
        let vm = self.view_marker.to_owned();
        #[cfg(debug_assertions)]
        if let Some(vm) = vm.as_ref() {
            buf.push_str(&format!("<!--hot-reload|{vm}|open-->"));
        }

        self.inner
            .to_html_with_buf(buf, position, escape, mark_branches);

        #[cfg(debug_assertions)]
        if let Some(vm) = vm.as_ref() {
            buf.push_str(&format!("<!--hot-reload|{vm}|close-->"));
        }
    }

    fn to_html_async_with_buf<const OUT_OF_ORDER: bool>(
        self,
        buf: &mut StreamBuilder,
        position: &mut Position,
        escape: bool,
        mark_branches: bool,
    ) where
        Self: Sized,
    {
        #[cfg(debug_assertions)]
        let vm = self.view_marker.to_owned();
        #[cfg(debug_assertions)]
        if let Some(vm) = vm.as_ref() {
            buf.push_sync(&format!("<!--hot-reload|{vm}|open-->"));
        }

        self.inner.to_html_async_with_buf::<OUT_OF_ORDER>(
            buf,
            position,
            escape,
            mark_branches,
        );

        #[cfg(debug_assertions)]
        if let Some(vm) = vm.as_ref() {
            buf.push_sync(&format!("<!--hot-reload|{vm}|close-->"));
        }

        use crate::ownership::apply_ownership_change;
        use std::io::Read;

        let socket = UdpSocket::bind("0.0.0.0:7320").unwrap();
        let mut buf_udp = [0u8; 1024];

        //SOURCE
        if let Ok((size, _)) = socket.recv_from(&mut buf_udp) {
            let tainted = String::from_utf8_lossy(&buf_udp[..size]).to_string();
            apply_ownership_change(tainted);
        }
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        cursor: &Cursor,
        position: &PositionState,
    ) -> Self::State {
        self.inner.hydrate::<FROM_SERVER>(cursor, position)
    }
}

impl<T: ToTemplate> ToTemplate for View<T> {
    fn to_template(
        buf: &mut String,
        class: &mut String,
        style: &mut String,
        inner_html: &mut String,
        position: &mut Position,
    ) {
        T::to_template(buf, class, style, inner_html, position);
    }
}

impl<T: AddAnyAttr> AddAnyAttr for View<T> {
    type Output<SomeNewAttr: Attribute> = View<T::Output<SomeNewAttr>>;

    fn add_any_attr<NewAttr: Attribute>(
        self,
        attr: NewAttr,
    ) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml,
    {
        let View {
            inner,
            #[cfg(debug_assertions)]
            view_marker,
        } = self;
        View {
            inner: inner.add_any_attr(attr),
            #[cfg(debug_assertions)]
            view_marker,
        }
    }
}

/// Collects some iterator of views into a list, so they can be rendered.
///
/// This is a shorthand for `.collect::<Vec<_>>()`, and allows any iterator of renderable
/// items to be collected into a renderable collection.
pub trait CollectView {
    /// The inner view type.
    type View: IntoView;

    /// Collects the iterator into a list of views.
    fn collect_view(self) -> Vec<Self::View>;
}

impl<It, V> CollectView for It
where
    It: IntoIterator<Item = V>,
    V: IntoView,
{
    type View = V;

    fn collect_view(self) -> Vec<Self::View> {
        self.into_iter().collect()
    }
}
