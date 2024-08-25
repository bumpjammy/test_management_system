use std::ptr;

// Lots of pointer arithmetic
// My own implementation of a vector, with useful functions for sorting, searching, etc

pub(crate) enum ComparisonResult { // The result of a comparison
    Equal,
    FirstGreater,
    SecondGreater,
}

// T is a generic type, allowing this vector to be used for any object
struct MyVector<T> {
    slice: *mut T, // Raw pointer to the data
    length: u32, // Length of the current data
    capacity: u32, // How much memory is allocated
    sorted: Option<fn(T, T) -> ComparisonResult>, // Optional comparison function for sorting
}

impl<T> MyVector<T> {
    // Create a new MyVector object with specified capacity
    fn new(capacity: u32) -> MyVector<T> {
        let layout = std::alloc::Layout::array::<T>(capacity as usize).unwrap(); // Allocate memory with the layout of an array
        let ptr = unsafe { // Unsafe code, as it deals with raw pointers
            std::alloc::alloc(layout) as *mut T // Get a raw pointer to the location of the array
        };

        MyVector {
            slice: ptr,
            length: 0,
            capacity,
            sorted: None,
        }
    }

    // Push an element to the end of MyVector
    fn push(&mut self, value: T) {
        if self.length < self.capacity { // Ensure there is enough room to add to the vector.
            unsafe { // Unsafe code, as it deals with raw pointers
                // Write the value to the next position in the slice
                std::ptr::write(self.slice, self.length as usize);
            }
            self.length += 1; // Increment length by 1
        } else {
            // TODO increase capacity
        }
    }

    // Remove an element at some index and returns the element if removed successfully
    fn remove(&mut self, index: u32) -> Option<T> {
        if index >= self.length { // Cannot remove, as it is out of bounds
            return None;
        }

        unsafe {
            let removed_element_ptr = self.slice.add(index as usize); // Get the correct pointer by adding from the start of the array
            let removed_element = ptr::read(removed_element_ptr); // Read the element from the pointer

            if(index as usize) < (self.length as usize - 1) { // If it is not the last element
                // Shift all elements after it back one space, to avoid gaps
                ptr::copy(
                    removed_element_ptr.add(1), // Where to move from
                    removed_element_ptr, // Where to move to (back one spot)
                    self.length as usize - index as usize - 1 // How many elements (until end of array)
                );
            }

            self.length -= 1; // Decrement length
            removed_element // Return removed element
        }
    }

    // Sets the value at a certain index, returns old value at that index if successful
    fn set(&mut self, index: u32) -> Option<T>

    // Get array as an immutable slice, useful for reading data from the array without editing
    fn as_slice(&self) -> &[T] { // Returns a slice (an array)
        unsafe {
            std::slice::from_raw_parts(self.slice, self.length as usize) // Convert our allocated memory into a slice
        }
    }

    // Get element at some index in an array
    fn get(&self, index: u32) -> Option<T> { // Returns optional, as element may not exist
        self.as_slice().get(index)
    }
}