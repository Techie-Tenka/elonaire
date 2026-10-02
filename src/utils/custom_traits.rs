use gloo_net::http::Headers;

pub trait EnumerableEnum: Sized {
    fn variants_slice() -> Vec<Self>;
}

pub trait IntoGlooHeaders {
    fn into_gloo_headers(self) -> Headers;
}

impl<I, K, V> IntoGlooHeaders for I
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn into_gloo_headers(self) -> Headers {
        let headers = Headers::new();
        for (k, v) in self {
            headers.append(k.as_ref(), v.as_ref());
        }
        headers
    }
}
