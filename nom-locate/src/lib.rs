pub struct Span<'s>(&'s str,&'s str);

impl<'s> Span<'s>{
    pub fn new(src:&'s str) -> Self{
        Self(src,src)
    }
}



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
