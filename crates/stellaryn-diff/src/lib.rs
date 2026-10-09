//! stellaryn-diff component boundary.
//!
//! Phase 1 intentionally contains no production parser, comparator, or renderer
//! implementation here. Later phases will add verified behavior behind this
//! crate boundary.

pub const COMPONENT: &str = "stellaryn-diff";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_name_is_stable() {
        assert_eq!(COMPONENT, "stellaryn-diff");
    }
}
