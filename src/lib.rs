#![allow(dead_code)]

mod node;
mod octree;

pub(crate) use node::Node;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
