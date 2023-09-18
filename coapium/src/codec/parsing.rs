pub fn take<const C: usize>(bytes: &[u8]) -> Result<(&[u8], [u8; C]), ()> {
    if bytes.len() < C {
        Err(())
    } else {
        let mut result = [0; C];
        result.copy_from_slice(&bytes[..C]);
        Ok((&bytes[C..], result))
    }
}

pub const fn many0<'a, I, P, T, E>(parser: P) -> impl Fn(&'a [I]) -> Result<(&'a [I], Vec<T>), E>
where
    P: Fn(&'a [I]) -> Result<(&'a [I], T), E>,
    I: 'a,
{
    move |bytes| {
        if bytes.is_empty() {
            return Ok((&[], vec![]));
        }

        let mut result = Vec::with_capacity(4);

        let mut rest = bytes;
        loop {
            match parser(rest) {
                Ok((left_over, parsed)) => {
                    rest = left_over;
                    result.push(parsed);
                }
                Err(_) => return Ok((&bytes[bytes.len() - rest.len()..], result)),
            }
        }
    }
}

pub fn single<T>(list: Vec<T>) -> Result<T, ()> {
    single_err(list, ())
}

pub fn single_err<T, E>(list: Vec<T>, err: E) -> Result<T, E> {
    single_or_err(list, || err)
}

pub fn single_or_err<T, F, E>(mut list: Vec<T>, err_fn: F) -> Result<T, E>
where
    F: FnOnce() -> E,
{
    match (list.pop(), list.pop()) {
        (Some(element), None) => Ok(element),
        _ => Err(err_fn()),
    }
}
