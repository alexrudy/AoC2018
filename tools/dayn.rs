use std::error::Error;

type Result<T> = ::std::result::Result<T, Box<Error>>;

pub(crate) fn main() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    
}