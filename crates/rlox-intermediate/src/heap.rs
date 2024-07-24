use std::collections::HashMap;

use crate::Reference;

pub struct Heap {
    allocated: Vec<Reference<()>>,
    string_pool: HashMap<String, Reference<String>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            allocated: Vec::new(),
            string_pool: HashMap::new(),
        }
    }

    pub fn spawn<T: 'static>(&mut self, value: T) -> Reference<T> {
        let reference = unsafe { Reference::new(Box::into_raw(Box::new(value))) };
        unsafe {
            self.allocated.push(reference.clone().cast());
        }
        reference
    }

    pub fn spawn_string(&mut self, value: String) -> Reference<String> {
        if let Some(reference) = self.string_pool.get(&value) {
            return reference.clone();
        }
        let reference = self.spawn(value.clone());
        self.string_pool.insert(value, reference.clone());
        reference
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        for allocation in &mut self.allocated {
            #[cfg(feature = "gc-sanitizer")]
            {
                println!("-- GC finalize: {allocation:?}")
            }
            unsafe { allocation.finalize() }
        }
    }
}
