//! Entity trait — the foundation for all domain objects in IronRoot.

/// Represents a domain object with a stable, unique identity.
///
/// Every persistent object in an IronRoot application should implement this
/// trait. The `id` method returns a reference to the object's identity value,
/// which must be unique within its aggregate root.
///
/// # Examples
///
/// ```rust
/// use ironroot_core::Entity;
///
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// impl Entity for User {
///     type Id = u64;
///
///     fn id(&self) -> &Self::Id {
///         &self.id
///     }
/// }
///
/// let user = User { id: 1, name: "Alice".to_string() };
/// assert_eq!(*user.id(), 1);
/// ```
pub trait Entity {
    /// The type used to uniquely identify this entity.
    type Id: Eq + std::fmt::Debug;

    /// Returns a reference to this entity's unique identifier.
    fn id(&self) -> &Self::Id;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Product {
        id: u32,
        #[allow(dead_code)]
        name: String,
    }

    impl Entity for Product {
        type Id = u32;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[test]
    fn entity_returns_correct_id() {
        let p = Product { id: 42, name: "Widget".to_string() };
        assert_eq!(*p.id(), 42);
    }
}
