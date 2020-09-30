#![no_std]

pub mod arbiter;
mod buffer;
mod selectable;
mod ingress;
mod delay;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
