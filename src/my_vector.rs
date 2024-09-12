use std::alloc::{alloc, dealloc, Layout};
use std::{alloc, io, ptr};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use rocket::tokio::fs::{create_dir_all, File};
use rocket::tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// Lots of pointer arithmetic :(
// My own implementation of a vector, with useful functions for sorting, searching, etc

#[derive(Clone)]
pub struct SafePointer<T> {
    inner: Arc<Mutex<*mut T>> // Thread-safe raw pointer
}

// SafePointer wrapper that allows for raw pointers to be sent across threads, which is necessary for the web server
impl<T> SafePointer<T> {
    pub fn new(ptr: *mut T) -> Self {
        SafePointer {
            inner: Arc::new(Mutex::new(ptr)),
        }
    }

    pub fn set_pointer(&self, new_pointer: *mut T) {
        let mut lock = self.inner.lock().unwrap();
        *lock = new_pointer;
    }

    pub fn get_pointer(&self) -> *mut T {
        let lock = self.inner.lock().unwrap();
        *lock
    }
}

// T is a generic type, allowing this vector to be used for any object
#[derive(Clone)]
pub struct MyVector<T> {
    slice: SafePointer<T>, // Raw pointer to the data
    pub length: u32, // Length of the current data, publicly accessible
    capacity: u32, // How much memory is allocated, maximum length
    sorted: bool, // Whether the array is sorted or not
}

impl<T> MyVector<T>
where
    T: Clone + PartialOrd, // Must be able to clone elements and compare elements for quick sort
    T: ToString + FromStr, // Must be able to convert to and from a string to read/write to a file
    T::Err: std::fmt::Debug, // Error must implement Debug for debugging purposes
{
    // Create a new MyVector object with specified capacity
    pub fn new_with_capacity(capacity: u32) -> MyVector<T> {
        let layout = Layout::array::<T>(capacity as usize).unwrap(); // Allocate memory with the layout of an array
        let ptr = unsafe { // Unsafe code, as it deals with raw pointers
            alloc::alloc(layout) as *mut T // Get a raw pointer to the location of the array
        };

        MyVector { // Create a new vector object
            slice: SafePointer::new(ptr),
            length: 0,
            capacity,
            sorted: false,
        }
    }

    // Default 10 elements capacity
    pub fn new() -> MyVector<T> {
        Self::new_with_capacity(10)
    }

    // Push an element to the end of MyVector
    pub async fn push(&mut self, value: T) {
        self.sorted = false;
        if self.length < self.capacity { // Ensure there is enough room to add to the vector.
            let slice = self.slice.get_pointer();
            
            unsafe { // Unsafe code, as it deals with raw pointers
                // Write the value to the next position in the slice
                ptr::write(slice.add(self.length as usize), value);
            }
            self.length += 1; // Increment length by 1
        } else {
            self.expand_to_length(self.capacity * 2).await; // Double max size of the array
            Box::pin(self.push(value)).await; // Retry push
        }
    }

    async fn expand_to_length(&mut self, length: u32) {
        // Lock the current slice (pointer) inside the Arc<Mutex<>>.
        let slice = self.slice.get_pointer();

        // Calculate the new layout for the desired length.
        let new_layout = match Layout::array::<T>(length as usize) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Failed to expand array! {:?}", e);
                panic!();
            }
        };

        unsafe {
            // Allocate new memory for the expanded slice.
            let new_ptr = alloc(new_layout) as *mut T;

            // Copy data from the old slice to the new slice.
            ptr::copy_nonoverlapping(slice, new_ptr, self.length as usize);

            // Calculate the old layout to deallocate the old slice.
            let old_layout = match Layout::array::<T>(self.capacity as usize) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Failed to deallocate memory! {:?}", e);
                    panic!();
                }
            };
            dealloc(slice as *mut u8, old_layout);

            // Update the slice and capacity with the new values.
            self.slice.set_pointer(new_ptr);
            self.capacity = length;
        }
    }

    // Remove an element at some index and returns the element if removed successfully
    pub async fn remove(&mut self, index: u32) -> Option<T> {
        if index >= self.length { // Cannot remove, as it is out of bounds
            panic!("Index out of bounds!")
        }

        let slice = self.slice.get_pointer();

        unsafe {
            let removed_element_ptr = slice.add(index as usize); // Get the correct pointer by adding from the start of the array
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
            Some(removed_element) // Return removed element
        }
    }

    // Sets the value at a certain index, returns old value at that index if successful
    pub async fn set(&mut self, index: u32, value: T) {
        if index >= self.length {
            panic!("Index out of bounds!")
        }

        self.sorted = false;

        let slice = self.slice.get_pointer();
        unsafe {
            let element_ptr = slice.add(index as usize);
            ptr::write(element_ptr, value);
        }
    }

    // Get array as an immutable slice, useful for reading data from the array without editing
    async fn as_slice(&self) -> &[T] { // Returns a slice (an array)
        let slice = self.slice.get_pointer();
        let slice_unguarded: *mut T = slice.clone();
        unsafe {
            std::slice::from_raw_parts(slice_unguarded, self.length as usize) // Convert our allocated memory into a slice
        }
    }

    // Get the value as a Clone
    pub async fn get(&self, index: u32) -> Option<T> { // Returns optional, as element may not exist
        let ptr = self.slice.get_pointer();
        let element_ptr = unsafe { ptr.add(index as usize) };

        // Safe read of the element
        unsafe {
            let value = if !element_ptr.is_null() {
                Some(ptr::read(element_ptr))
            } else {
                None
            };

            if let Some(v) = value.clone() {
                ptr::write(element_ptr, v);
            }

            value
        }
    }

    // Get element at some index in an array as a mutable reference
    pub async fn get_mut(&self, index: u32) -> Option<&mut T> { // Returns optional, as element may not exist

        if index > self.length {
            return None;
        }

        let ptr = self.slice.get_pointer();
        let element_ptr = unsafe { ptr.add(index as usize) };

        // Safe read of the element
        unsafe {
            let value = if !element_ptr.is_null() {
                ptr
            } else {
                return None;
            };

            Some(&mut *value)
        }
    }

    // Begins performing the quick sort
    pub async fn quick_sort(&mut self) {
        if self.length > 1 { // If there is more than one element, else nothing needs sorting
            let len = self.length;
            self.quick_sort_helper(0, len - 1).await; // Sort the whole array, to start with
        }
        self.sorted = true; // List is now sorted
    }

    // Helper method to perform quicksort on the vector between 2 indices
    async fn quick_sort_helper(&mut self, low: u32, high: u32) {
        if low < high { // If they are the same, sorting is not necessary
            let pivot_index = self.partition(low, high).await; // Get the pivot by partitioning the array
            if pivot_index > 0 { // If pivot is 0, there are no elements to sort on this side of the array
                Box::pin(self.quick_sort_helper(low, pivot_index - 1)).await; // If pivot is not 0, we still need to sort, continue
            }
            Box::pin(self.quick_sort_helper(pivot_index + 1, high)).await; // Sort other half of the array
        }
    }

    // Partitions the array into 2 parts, and returns the pivot index
    async fn partition(&mut self, low: u32, high: u32) -> u32 {
        let pivot_value = self.get(high).await; // Pivot element is the element at the high index
        let mut i = low; // Keep track of where the pivot element should be at the end

        for j in low..high { // Loop through the array from low to high - 1
            if self.get(j).await <= pivot_value.clone() { // If the current element is less than or equal to the pivot, swap them
                self.swap(i, j).await;
                i += 1; // Increment i
            }
        }

        self.swap(i, high).await; // Place pivot element into correct position, kept track of with i
        i // Return the pivot index
    }

    // Swap two elements in the vector
    async fn swap(&mut self, i: u32, j: u32) {
        let temp = match self.get(i).await {
            Some(value) => value,
            None => {
                eprintln!("Failed to swap values!");
                panic!();
            }
        }.clone();
        self.set(i, match self.get(j).await {
            Some(result) => result,
            None => {
                eprintln!("Failed to swap values!");
                panic!();
            }
        }.clone()).await;
        self.set(j, temp).await;
    }

    // Insert an element into a specific index
    pub async fn insert(&mut self, index: u32, value: T) {
        if index > self.length { // Cannot insert out of bounds
            panic!("Index out of bounds!");
        }

        if self.length >= self.capacity {
            self.expand_to_length(self.capacity * 2).await; // Double the capacity if necessary
        }

        let slice = self.slice.get_pointer();

        unsafe {
            ptr::copy( // Shift elements to the right to make space for the new element
                slice.add(index as usize),
                slice.add(index as usize + 1),
                (self.length - index) as usize,
            );

            ptr::write(slice.add(index as usize), value); // Insert the new element
        }

        self.length += 1; // Increment length
        self.sorted = false;
    }

    // Insert an element into the vector in a sorted order
    pub async fn insert_sorted(&mut self, value: T) {
        if !self.sorted {
            self.quick_sort().await; // Sort the vector if it's not already sorted
        }

        // Linear search for where to insert
        let mut index = 0;
        while index < self.length && match self.get(index).await {
            Some(value) => value,
            None => {
                eprintln!("Failed to insert value!");
                panic!();
            }
        } < value {
            index += 1;
        }

        self.insert(index, value).await;

        self.sorted = true; // Array is now sorted
    }

    // Perform a linear search
    pub async fn search<F>(&self, predicate: F) -> Option<u32>
    where
        F: Fn(&T) -> bool,
    {
        for i in 0..self.length { // Loop through the array
            let current_value = match self.get(i).await {
                Some(value) => value,
                None => {
                    eprintln!("Failed to search array!");
                    panic!();
                }
            };
            if predicate(&current_value) { // Use the predicate function to check if found
                return Some(i);
            }
        }

        None // Not in list, return None
    }

    // Perform a binary search
    pub async fn binary_search(&self, value: T) -> Option<u32> {
        if !self.sorted { // Must be sorted
            panic!("Vector is not sorted!");
        }

        let mut low = 0;
        let mut high = self.length as i32 - 1; // Low and high are start and end of vector

        while low <= high { // When low > high, element is not in list
            let mid = (low + high) / 2; // Get midpoint
            let mid_value = match self.get(mid as u32).await {
                Some(value) => value,
                None => {
                    eprintln!("Failed to search array!");
                    panic!();
                }
            }; // Get mid value

            if mid_value == value { // Value found
                return Some(mid as u32);
            } else if mid_value < value { // Value is to the right
                low = mid + 1;
            } else { // Value is to the left
                high = mid - 1;
            }
        }

        None
    }

    // Save the vector data to a specified file
    pub async fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let path = Path::new(file_path);

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }

        let mut file = File::create(&path).await?; // Ensure the file is mutable

        for i in 0..self.length { // Loop through elements of the vector
            let element_str = match self.get(i).await { // Get element
                Some(value) => value, // Should succeed, since we are looping through the array
                None => {
                    eprintln!("Failed to save vector to file!"); // Something has gone horribly wrong
                    panic!();
                }
            }.to_string(); // Convert the element to a string to write to file
            match file.write(format!("{}{}", element_str, "\n").as_bytes()).await { // Write to the file
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to save vector to file!");
                    panic!();
                }
            };
        }

        Ok(())
    }

    // Load vector data from a specified file
    pub async fn load_from_file(file_path: &str) -> MyVector<T> {
        let path = Path::new(file_path);

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            match create_dir_all(parent).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to load vector from file! {:?}", e);
                    panic!();
                }
            };
        }

        // Create the file if it does not exist
        if !path.exists() {
            File::create(path).await.unwrap();
        }

        let path = Path::new(file_path);
        let file = File::open(&path).await.unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut vec = MyVector::new_with_capacity(10); // Default capacity

        while let Ok(Some(line)) = lines.next_line().await {
            match T::from_str(&line) {
                Ok(value) => vec.push(value).await,
                Err(_) => {}
            }
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::MyVector;
    use std::fs;
    use rocket::tokio;

    #[tokio::test]
    async fn test_new_with_capacity() {
        let vec: MyVector<i32> = MyVector::new_with_capacity(5);
        assert_eq!(vec.capacity, 5);
        assert_eq!(vec.length, 0);
        assert_eq!(vec.sorted, false);
    }

    #[tokio::test]
    async fn test_new_default_capacity() {
        let vec: MyVector<i32> = MyVector::new();
        assert_eq!(vec.capacity, 10);
        assert_eq!(vec.length, 0);
        assert_eq!(vec.sorted, false);
    }

    #[tokio::test]
    async fn test_push_within_capacity() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        assert_eq!(vec.length, 3);
        assert_eq!(vec.capacity, 3);
        assert_eq!(vec.get(0).await.unwrap(), 1);
        assert_eq!(vec.get(1).await.unwrap(), 2);
        assert_eq!(vec.get(2).await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_push_expand_capacity() {
        let mut vec = MyVector::new_with_capacity(2);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await; // Should trigger expansion

        assert_eq!(vec.length, 3);
        assert_eq!(vec.capacity, 4); // Capacity should double
        assert_eq!(vec.get(0).await.unwrap(), 1);
        assert_eq!(vec.get(1).await.unwrap(), 2);
        assert_eq!(vec.get(2).await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_remove() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        let removed = vec.remove(1).await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(vec.length, 2);
        assert_eq!(vec.get(0).await.unwrap(), 1);
        assert_eq!(vec.get(1).await.unwrap(), 3);
    }

    #[tokio::test]
    #[should_panic(expected = "Index out of bounds!")]
    async fn test_remove_out_of_bounds() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        vec.remove(5).await; // This should panic
    }

    #[tokio::test]
    async fn test_set() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        vec.set(1, 5).await;
        assert_eq!(vec.get(1).await.unwrap(), 5);
        assert_eq!(vec.sorted, false);
    }

    #[tokio::test]
    #[should_panic(expected = "Index out of bounds!")]
    async fn test_set_out_of_bounds() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        vec.set(5, 10).await; // This should panic
    }

    #[tokio::test]
    async fn test_quick_sort() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(3).await;
        vec.push(1).await;
        vec.push(4).await;
        vec.push(5).await;
        vec.push(2).await;

        vec.quick_sort().await;
        assert_eq!(vec.as_slice().await, &[1, 2, 3, 4, 5]);
        assert!(vec.sorted);
    }

    #[tokio::test]
    async fn test_swap() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        vec.swap(0, 2).await;
        assert_eq!(vec.get(0).await.unwrap(), 3);
        assert_eq!(vec.get(2).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_insert_in_place() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(4).await;
        vec.push(5).await;

        vec.insert(2, 3).await; // Insert 3 at index 2

        assert_eq!(vec.length, 5);
        assert_eq!(vec.as_slice().await, &[1, 2, 3, 4, 5]);
        assert_eq!(vec.sorted, false); // Inserting an element disrupts sorting
    }

    #[tokio::test]
    #[should_panic(expected = "Index out of bounds!")]
    async fn test_insert_in_place_out_of_bounds() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;

        vec.insert(5, 4).await; // This should panic, index is out of bounds
    }

    #[tokio::test]
    async fn test_insert_in_place_expand_capacity() {
        let mut vec = MyVector::new_with_capacity(3);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(4).await;

        vec.insert(2, 3).await; // This should trigger expansion and insert at index 2

        assert_eq!(vec.length, 4);
        assert_eq!(vec.capacity, 6); // Capacity should double
        assert_eq!(vec.as_slice().await, &[1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_insert_sorted() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(3).await;
        vec.push(5).await;
        vec.sorted = true; // Indicate that the vector is already sorted

        vec.insert_sorted(4).await; // Insert 4 into its sorted position

        assert_eq!(vec.length, 4);
        assert_eq!(vec.as_slice().await, &[1, 3, 4, 5]);
        assert!(vec.sorted); // Vector should still be sorted
    }

    #[tokio::test]
    async fn test_insert_sorted_unsorted_vector() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(5).await;
        vec.push(1).await;
        vec.push(3).await;

        vec.insert_sorted(4).await; // Insert 4 and sort the vector

        assert_eq!(vec.length, 4);
        assert_eq!(vec.as_slice().await, &[1, 3, 4, 5]);
        assert!(vec.sorted); // Vector should now be sorted
    }

    #[tokio::test]
    async fn test_insert_sorted_at_start() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(2).await;
        vec.push(3).await;
        vec.push(4).await;
        vec.sorted = true;

        vec.insert_sorted(1).await; // Insert 1 at the start

        assert_eq!(vec.length, 4);
        assert_eq!(vec.as_slice().await, &[1, 2, 3, 4]);
        assert!(vec.sorted);
    }

    #[tokio::test]
    async fn test_insert_sorted_at_end() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;
        vec.sorted = true;

        vec.insert_sorted(4).await; // Insert 4 at the end

        assert_eq!(vec.length, 4);
        assert_eq!(vec.as_slice().await, &[1, 2, 3, 4]);
        assert!(vec.sorted);
    }

    #[tokio::test]
    async fn test_binary_search_found() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(3).await;
        vec.push(4).await;
        vec.push(5).await;
        vec.sorted = true;

        assert_eq!(vec.binary_search(3).await, Some(2)); // Index of element 3 should be 2
        assert_eq!(vec.binary_search(1).await, Some(0)); // Index of element 1 should be 0
        assert_eq!(vec.binary_search(5).await, Some(4)); // Index of element 5 should be 4
    }

    #[tokio::test]
    async fn test_binary_search_not_found() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(1).await;
        vec.push(2).await;
        vec.push(4).await;
        vec.push(5).await;
        vec.sorted = true;

        assert_eq!(vec.binary_search(3).await, None); // Element 3 is not in the vector
        assert_eq!(vec.binary_search(0).await, None); // Element 0 is not in the vector
        assert_eq!(vec.binary_search(6).await, None); // Element 6 is not in the vector
    }

    #[tokio::test]
    #[should_panic(expected = "Vector is not sorted!")]
    async fn test_binary_search_unsorted_vector() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(5).await;
        vec.push(1).await;
        vec.push(3).await;

        vec.binary_search(3).await; // This should panic, as the vector is not sorted
    }

    #[tokio::test]
    async fn test_binary_search_empty_vector() {
        let mut vec: MyVector<i32> = MyVector::new_with_capacity(5);
        vec.quick_sort().await;

        assert_eq!(vec.binary_search(1).await, None); // Should return None for an empty vector
    }

    #[tokio::test]
    async fn test_linear_search_found() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(10).await;
        vec.push(20).await;
        vec.push(30).await;
        vec.push(40).await;
        vec.push(50).await;

        assert_eq!(vec.search(|a| *a == 30).await, Some(2)); // Element 30 is at index 2
        assert_eq!(vec.search(|a| *a == 10).await, Some(0)); // Element 10 is at index 0
        assert_eq!(vec.search(|a| *a == 50).await, Some(4)); // Element 50 is at index 4
    }

    #[tokio::test]
    async fn test_linear_search_not_found() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(10).await;
        vec.push(20).await;
        vec.push(30).await;
        vec.push(40).await;
        vec.push(50).await;

        assert_eq!(vec.search(|a| *a == 25).await, None); // Element 25 is not in the vector
        assert_eq!(vec.search(|a| *a == 5).await, None);  // Element 5 is not in the vector
        assert_eq!(vec.search(|a| *a == 60).await, None); // Element 60 is not in the vector
    }

    #[tokio::test]
    async fn test_linear_search_empty_vector() {
        let vec: MyVector<i32> = MyVector::new_with_capacity(5);

        assert_eq!(vec.search(|a| *a == 10).await, None); // Searching in an empty vector should return None
    }

    #[tokio::test]
    async fn test_linear_search_multiple_occurrences() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(10).await;
        vec.push(20).await;
        vec.push(20).await;
        vec.push(30).await;

        assert_eq!(vec.search(|a| *a == 20).await, Some(1)); // Should return the first occurrence at index 1
    }

    #[tokio::test]
    async fn test_linear_search_last_element() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(10).await;
        vec.push(20).await;
        vec.push(30).await;
        vec.push(40).await;
        vec.push(50).await;

        assert_eq!(vec.search(|a| *a == 50).await, Some(4)); // Searching for the last element, should return index 4
    }


    #[tokio::test]
    async fn test_save_to_file_and_load_from_file() {
        let mut vec = MyVector::new_with_capacity(5);
        vec.push(10).await;
        vec.push(20).await;
        vec.push(30).await;

        let file_path = "test_vector.txt";

        // Save the vector to a file
        vec.save_to_file(file_path).await.expect("Failed to save to file");

        // Load the vector from the file
        let loaded_vec = MyVector::<i32>::load_from_file(file_path).await;

        assert_eq!(loaded_vec.as_slice().await, vec.as_slice().await);

        // Clean up the test file
        fs::remove_file(file_path).expect("Failed to delete test file");
    }

    #[tokio::test]
    async fn test_load_from_file_invalid_data() {
        let file_path = "test_invalid_data.txt";

        // Create a file with invalid data
        let invalid_data = "10\n20\ninvalid data\n30";
        fs::write(file_path, invalid_data).expect("Failed to write test file");

        // Load the vector from the file
        let loaded_vec = MyVector::<i32>::load_from_file(file_path).await;

        assert_eq!(loaded_vec.as_slice().await, &[10, 20, 30]); // Invalid data should be skipped

        // Clean up the test file
        fs::remove_file(file_path).expect("Failed to delete test file");
    }
}
