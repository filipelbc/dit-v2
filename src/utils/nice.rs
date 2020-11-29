pub trait Nice {
    fn nice(&self) -> String;
}

impl<T> Nice for Option<T>
    where T: Nice
{
    fn nice(&self) -> String {
        match self {
            Some(x) => x.nice(),
            None => String::new(),
        }
    }
}
