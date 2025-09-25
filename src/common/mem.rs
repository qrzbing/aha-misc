//! Memory tools

use libafl_bolts::{
    AsSlice, AsSliceMut,
    shmem::{ShMemDescription, ShMemProvider, UnixShMem, UnixShMemProvider},
};
use std::alloc::{Layout, alloc};

unsafe extern "C" {
    /// Compare two memory maps to detect new bits.
    ///
    /// # Arguments
    /// * `virgin_map` - Pointer to the virgin (baseline) map.
    /// * `shm_map` - Pointer to the shared memory map to compare against.
    /// * `map_size` - Size of the map in bytes.
    ///
    /// # Returns
    /// A byte value indicating if new bits were found:
    /// - 0: No new bits found
    /// - 1: New hitcount found
    /// - 2: New tuples found
    pub fn has_new_bits(virgin_map: *mut u8, shm_map: *mut u8, map_size: u32) -> u8;
}

/// Allocate memory with the specified size and alignment.
///
/// # Arguments
/// * `size` - The size of the memory to allocate.
/// * `align` - The alignment requirement for the memory.
///
/// # Returns
/// A pointer to the allocated memory.
#[inline]
pub fn get_layout(size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align).unwrap();
    let layout = unsafe { alloc(layout) as *mut u8 };
    if layout.is_null() {
        panic!("Failed to alloc layout");
    }
    layout
}

/// Read a value of type T from shared memory at the specified offset.
///
/// # Arguments
/// * `shm` - Reference to the Unix shared memory.
/// * `offset` - Offset in bytes from the start of the shared memory.
///
/// # Returns
/// The value of type T at the specified offset.
///
/// # Type Parameters
/// * `T` - The type of value to read, which must be `Copy`.
#[inline]
pub fn read_from_mem<T>(shm: &UnixShMem, offset: usize) -> T
where
    T: Copy,
{
    let slice = shm.as_slice();
    unsafe { *(slice.as_ptr().add(offset) as *const T) }
}

/// Write a value of type T to shared memory at the specified offset.
///
/// # Arguments
/// * `shm` - Mutable reference to the Unix shared memory.
/// * `value` - The value to write.
/// * `offset` - Offset in bytes from the start of the shared memory.
///
/// # Type Parameters
/// * `T` - The type of value to write, which must be `Copy`.
#[inline]
pub fn write_to_mem<T>(shm: &mut UnixShMem, value: T, offset: usize)
where
    T: Copy,
{
    let slice = shm.as_slice_mut();
    unsafe {
        *(slice.as_mut_ptr().add(offset) as *mut T) = value;
    }
}

/// Read a value of type T from a raw pointer at the specified offset.
///
/// # Arguments
/// * `ptr` - Raw pointer to the memory region.
/// * `offset` - Offset in bytes from the start of the memory region.
///
/// # Returns
/// The value of type T at the specified offset.
///
/// # Type Parameters
/// * `T` - The type of value to read, which must be `Copy`.
#[inline]
pub fn read_from_ptr<T>(ptr: *const u8, offset: usize) -> T
where
    T: Copy,
{
    unsafe { *(ptr.add(offset) as *const T) }
}

/// Write a value of type T to a raw pointer at the specified offset.
///
/// # Arguments
/// * `ptr` - Raw pointer to the memory region.
/// * `offset` - Offset in bytes from the start of the memory region.
/// * `value` - The value to write.
///
/// # Type Parameters
/// * `T` - The type of value to write, which must be `Copy`.
#[inline]
pub fn write_to_ptr<T>(ptr: *mut u8, offset: usize, value: T)
where
    T: Copy,
{
    unsafe {
        *(ptr.add(offset) as *mut T) = value;
    }
}

/// Display the memory layout of a shared memory segment.
///
/// # Arguments
/// * `id_str` - The identifier string for the shared memory.
/// * `size` - The size of the shared memory to display.
///
/// # Description
/// This function opens a shared memory segment by its identifier and
/// displays its contents in a hexadecimal and ASCII format.
pub fn show_shm_mem_layout(id_str: &str, size: usize) {
    let mut shm_provider = UnixShMemProvider::new().unwrap();
    let mut shmem = shm_provider
        .shmem_from_description(ShMemDescription::from_string_and_size(id_str, size))
        .unwrap();
    let shmem_ptr = shmem.as_slice_mut().as_mut_ptr();
    show_shm_ptr_layout(shmem_ptr, size);
}

/// Display the memory layout at a specific memory pointer.
///
/// # Arguments
/// * `ptr` - Pointer to the memory region to display.
/// * `size` - The size of the memory region to display.
///
/// # Description
/// This function displays the contents of a memory region in both
/// hexadecimal and ASCII format, with 8 bytes per line. Non-printable
/// ASCII characters are displayed as periods.
pub fn show_shm_ptr_layout(ptr: *mut u8, size: usize) {
    println!("shm_addr: {:p}", ptr);
    println!("Memory contents (showing {} bytes):", size);

    // Print hexadecimal values
    for i in 0..size {
        unsafe {
            // Print hexadecimal value
            print!("{:02x} ", *ptr.add(i));

            // Line break every 8 bytes
            if (i + 1) % 8 == 0 {
                print!("  ");

                // Print corresponding ASCII characters (if printable)
                for j in (i - 7)..=i {
                    let c = *ptr.add(j);
                    print!("{}", if c >= 32 && c <= 126 { c as char } else { '.' });
                }

                println!();
            }
        }
    }

    // If the last line is not complete (less than 8 bytes), pad and print ASCII
    if size % 8 != 0 {
        // Calculate padding spaces needed
        let padding = (8 - (size % 8)) * 3;
        for _ in 0..padding {
            print!(" ");
        }

        print!("  ");

        // Print ASCII for the last line
        let start = size - (size % 8);
        for j in start..size {
            unsafe {
                let c = *ptr.add(j);
                print!("{}", if c >= 32 && c <= 126 { c as char } else { '.' });
            }
        }

        println!();
    }
}
