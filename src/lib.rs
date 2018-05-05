//! This crate provides special handling for reference primitive type.
//!
//! We know that the value a reference pointers to is kept alive during the full
//! lifetime of the reference.
//!
//! Many times a mutable reference is consumed by some specific handling,
//! but they didn't bother return the mutable reference back to the caller even
//! if they didn't give meaningful results. At most time this is fine, since you can
//! just recreate the reference by borrow again.
//! However there is a problem when the reference comes from an argument.
//!
//! This crate catches this pattern and provide it as a trait `TryTransform`,
//! making it possible without any NLL support.
//!
//! ```
//! # use std::hash::Hash;
//! # use std::collections::HashMap;
//! #
//! use try_transform_mut::TryTransform;
//! 
//! fn get_default<'r, K: Hash + Eq + Copy, V: Default>(
//!     map: &'r mut HashMap<K, V>,
//!     key: K,
//! ) -> &'r mut V {
//!     match map.try_transform(|m| m.get_mut(&key)) {
//!         Ok(value) => {
//!             value
//!         }
//!         Err(map) => {
//!             map.insert(key, V::default());
//!             map.get_mut(&key).unwrap()
//!         }
//!     }
//! }
//! ```
//!

/// A trait providing `try_transform` method to reference primitive type.
pub trait TryTransform {
    /// Try to consume a reference and transform it to `Some(B)` with a closure.
    /// If failed and returned `None`, return the original reference value.
    ///
    /// Especially useful for mutable references.
    fn try_transform<B, F>(self, f: F) -> Result<B, Self>
    where
        Self: Sized,
        F: FnOnce(Self) -> Option<B>;
}

impl<'a, T> TryTransform for &'a T {
    fn try_transform<B, F>(self, f: F) -> Result<B, Self>
    where
        Self: Sized,
        F: FnOnce(Self) -> Option<B>,
    {
        if let Some(v) = f(self) {
            return Ok(v);
        }

        Err(self)
    }
}

impl<'a, T> TryTransform for &'a mut T {
    fn try_transform<B, F>(self, f: F) -> Result<B, Self>
    where
        Self: Sized,
        F: FnOnce(Self) -> Option<B>,
    {
        let this: *mut T = self as _;

        if let Some(v) = f(self) {
            return Ok(v);
        }

        Err(unsafe { this.as_mut().unwrap() })
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;
    use std::collections::HashMap;

    use super::TryTransform;

    fn get_default<'r, K: Hash + Eq + Copy, V: Default>(
        map: &'r mut HashMap<K, V>,
        key: K,
    ) -> &'r mut V {
        match map.try_transform(|m| m.get_mut(&key)) {
            Ok(value) => value,
            Err(map) => {
                map.insert(key, V::default());
                map.get_mut(&key).unwrap()
            }
        }
    }

    #[test]
    fn it_works() {
        let mut a: HashMap<usize, usize> = HashMap::new();
        get_default(&mut a, 2);
    }
}
