pub fn from_bool<T>(b: bool) -> impl FnOnce(T) -> Option<T> {
    move |v: T| if b { Some(v) } else { None }
}
