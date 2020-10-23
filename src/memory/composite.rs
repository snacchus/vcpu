use crate::{Storage, StorageMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

// TODO: think about providing a more specialized version of this
//       with a single chunk of fixed, main memory and a collection of
//       custom devices

/// Error type for [`CompositeMemory.mount`].
///
/// [`CompositeMemory.mount`]: ./struct.CompositeMemory.html#method.mount
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MountError {
    /// The mount operation failed because it would have resulted in intersecting fragments.
    FragmentIntersection,

    /// The mount operation failed because another fragment has already been mounted with the same key.
    KeyAlreadyExists,
}

type AdressedFragment = (u32, Box<dyn StorageMut>);

/// Represents a [`StorageMut`] which consists of "fragments" instead of one contiguous block of memory.
///
/// Fragments are simply represented by [`StorageMut`] trait objects and can be "mounted" at a certain address.
/// This can be used to simulate hardware devices which are bound to certain address ranges, without wasting physical memory on unused ranges.
///
/// The length of a `CompositeMemory` is equal to the last fragment's address plus its length.
///
/// Nesting `CompositeMemory` objects is possible, though not recommended since the speed of lookups will suffer.
/// Flattening the nested objects into one `CompositeMemory` instance is preferable.
///
/// [`StorageMut`]: ../trait.StorageMut.html
#[derive(Default)]
pub struct CompositeMemory {
    fragments: Vec<AdressedFragment>,
    registry: HashMap<String, usize>,
}

impl CompositeMemory {
    /// Constructs an empty `CompositeMemory` object.
    pub fn new() -> CompositeMemory {
        Default::default()
    }

    /// Mounts the given `fragment` at the specified `address` and registers it with the specified `key`.
    ///
    /// The `fragment` will occupy the address range `[address..address+fragment.length()]`.
    ///
    /// # Errors
    /// Returns an error if another fragment has already been mounted using the specified `key`.
    ///
    /// Returns an error if mounting the `fragment` at the specified `address` would lead to
    /// an intersection with another, already mounted fragment.
    ///
    /// # Panics
    /// Panics if `address + fragment.length()` results in integer overflow, i.e. would be greater than `u32::max_value()`.
    ///
    /// # Examples
    /// Successful mount:
    /// ```
    /// use vcpu::{CompositeMemory, Storage};
    ///
    /// let mut memory = CompositeMemory::new();
    /// assert_eq!(
    ///     memory.mount(16, "f0", [0u8, 1u8, 2u8, 3u8]),
    ///     Ok(())
    /// );
    /// assert_eq!(memory.read_word(16), Ok(50462976));
    /// assert_eq!(memory.read_byte(0), Err(()));
    /// ```
    ///
    /// Consecutive fragments:
    /// ```
    /// use vcpu::{CompositeMemory, MountError, Storage};
    ///
    /// let mut memory = CompositeMemory::new();
    /// assert_eq!(
    ///     memory.mount(0, "f0", [0u8; 16]),
    ///     Ok(())
    /// );
    /// assert_eq!(
    ///     memory.mount(16, "f1", [0u8; 16]),
    ///     Ok(())
    /// );
    /// ```
    ///
    /// Intersecting fragments:
    /// ```
    /// use vcpu::{CompositeMemory, MountError, Storage};
    ///
    /// let mut memory = CompositeMemory::new();
    /// assert_eq!(
    ///     memory.mount(0, "f0", [0u8; 16]),
    ///     Ok(())
    /// );
    /// assert_eq!(
    ///     memory.mount(15, "f1", [0u8; 16]),
    ///     Err(MountError::FragmentIntersection)
    /// );
    /// ```
    pub fn mount<S: StorageMut + 'static>(
        &mut self,
        address: u32,
        key: &str,
        fragment: S,
    ) -> Result<(), MountError> {
        if self.registry.contains_key(key) {
            return Err(MountError::KeyAlreadyExists);
        }

        let upper_bound = address
            .checked_add(fragment.length())
            .expect("Fragment upper bound exceeds valid address range.");
        let index = self.find_mount_index(address, upper_bound)?;

        self.fragments.insert(index, (address, Box::new(fragment)));
        self.registry.insert(key.to_string(), index);

        Ok(())
    }

    /// Looks for a fragment mounted as `key` and if found, unmounts and returns it as `Some`.
    /// Returns `None` if no such fragment was found.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{CompositeMemory, MountError, Storage};
    ///
    /// let mut memory = CompositeMemory::new();
    /// assert_eq!(
    ///     memory.mount(0, "f0", [0u8; 16]),
    ///     Ok(())
    /// );
    /// assert!(memory.unmount("something").is_none());
    /// assert!(memory.unmount("f0").is_some());
    /// assert!(memory.unmount("f0").is_none());
    /// ```
    pub fn unmount(&mut self, key: &str) -> Option<Box<dyn StorageMut>> {
        self.registry
            .remove(key)
            .map(|i| self.fragments.remove(i).1)
    }

    fn find_mount_index(&self, address: u32, upper_bound: u32) -> Result<usize, MountError> {
        for (i, (frag_addr, frag)) in self.fragments.iter().enumerate() {
            let frag_upper = frag_addr + frag.length();
            if *frag_addr >= address {
                return if upper_bound > *frag_addr {
                    Err(MountError::FragmentIntersection)
                } else {
                    Ok(i)
                };
            } else if frag_upper > address {
                return Err(MountError::FragmentIntersection);
            }
        }
        Ok(self.fragments.len())
    }

    fn get_index(&self, address: u32) -> Option<usize> {
        match self.fragments.binary_search_by_key(&address, |e| e.0) {
            Ok(i) => Some(i),
            Err(i) => {
                if i > 0 {
                    Some(i - 1)
                } else {
                    None
                }
            }
        }
    }

    fn get_fragment(&self, address: u32) -> Option<(&dyn StorageMut, u32)> {
        let index = self.get_index(address)?;
        if index >= self.fragments.len() {
            return None;
        }

        let (frag_addr, fragment) = &self.fragments[index];
        Some((fragment.deref(), address - frag_addr))
    }

    fn get_fragment_mut(&mut self, address: u32) -> Option<(&mut dyn StorageMut, u32)> {
        let index = self.get_index(address)?;
        if index >= self.fragments.len() {
            return None;
        }

        let (frag_addr, fragment) = &mut self.fragments[index];
        Some((fragment.deref_mut(), address - *frag_addr))
    }
}

impl Storage for CompositeMemory {
    fn length(&self) -> u32 {
        if !self.fragments.is_empty() {
            let (address, frag) = &self.fragments[self.fragments.len() - 1];
            address + frag.length()
        } else {
            0
        }
    }

    fn check_range(&self, address: u32, length: u32) -> bool {
        if let Some((fragment, local_address)) = self.get_fragment(address) {
            fragment.check_range(local_address, length)
        } else {
            false
        }
    }

    fn read(&self, address: u32, size: u32) -> Result<u32, ()> {
        let (fragment, local_address) = self.get_fragment(address).ok_or(())?;
        fragment.read(local_address, size)
    }
}

impl StorageMut for CompositeMemory {
    fn write(&mut self, address: u32, size: u32, value: u32) -> Result<(), ()> {
        let (fragment, local_address) = self.get_fragment_mut(address).ok_or(())?;
        fragment.write(local_address, size, value)
    }
}

#[test]
fn find_mount_index() {
    let mut memory = CompositeMemory::new();
    assert_eq!(memory.find_mount_index(0, 16), Ok(0));
    assert_eq!(memory.mount(0, "f0", [0u8; 16]), Ok(()));
    assert_eq!(
        memory.find_mount_index(8, 24),
        Err(MountError::FragmentIntersection)
    );
    assert_eq!(memory.mount(20, "f1", [0u8; 16]), Ok(()));
    assert_eq!(memory.find_mount_index(16, 20), Ok(1));
    assert_eq!(memory.find_mount_index(18, 20), Ok(1));
    assert_eq!(memory.find_mount_index(40, 44), Ok(2));
    assert_eq!(
        memory.find_mount_index(15, 20),
        Err(MountError::FragmentIntersection)
    );
    assert_eq!(
        memory.find_mount_index(16, 21),
        Err(MountError::FragmentIntersection)
    );
}

#[test]
fn access_mounted_fragment() {
    let mut comp = CompositeMemory::new();
    assert_eq!(comp.mount(0, "f0", vec![0u8; 64]), Ok(()));
    assert_eq!(comp.mount(0xF1ED_0000, "f1", vec![0u8; 1]), Ok(()));
    assert_eq!(comp.write_byte(0xF1ED_0001, 1), Err(()));
    assert_eq!(comp.write_byte(0xF1ED_0000, 1), Ok(()));
    let unmount_result = comp.unmount("f1");
    assert!(unmount_result.is_some());
    let fragment = unmount_result.unwrap();
    assert_eq!(fragment.read_byte(0), Ok(1));
}
