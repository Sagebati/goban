mod goban;

#[cfg(test)]
mod tests {
    use crate::goban::{Goban, SizeGoban};
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn goban(){
        Goban::new(SizeGoban::Nineteen as usize);
        assert_eq!(true,true)
    }
}
