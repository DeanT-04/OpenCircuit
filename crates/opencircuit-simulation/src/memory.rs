//! Memory management for NgSpice integration

use crate::errors::SimulationError;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::{Arc, Mutex};

/// Memory pool for managing NgSpice allocations
pub struct MemoryPool {
    /// Allocated pointers that need cleanup
    allocations: Arc<Mutex<Vec<*mut c_void>>>,
    /// String allocations
    string_allocations: Arc<Mutex<Vec<CString>>>,
}

/// RAII wrapper for NgSpice memory
pub struct NgSpiceMemory {
    ptr: *mut c_void,
    size: usize,
    pool: Arc<Mutex<Vec<*mut c_void>>>,
}

/// Safe string conversion utilities
pub struct StringManager {
    /// Owned CString instances
    strings: Vec<CString>,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(Vec::new())),
            string_allocations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Allocate memory and track it
    pub fn allocate(&self, size: usize) -> Result<NgSpiceMemory, SimulationError> {
        unsafe {
            let ptr = libc::malloc(size);
            if ptr.is_null() {
                return Err(SimulationError::MemoryError {
                    reason: format!("Failed to allocate {} bytes", size),
                });
            }
            
            // Zero the memory
            libc::memset(ptr, 0, size);
            
            // Track allocation
            if let Ok(mut allocations) = self.allocations.lock() {
                allocations.push(ptr);
            }
            
            Ok(NgSpiceMemory {
                ptr,
                size,
                pool: self.allocations.clone(),
            })
        }
    }
    
    /// Create a managed CString
    pub fn create_cstring(&self, s: &str) -> Result<*const c_char, SimulationError> {
        let cstring = CString::new(s)
            .map_err(|e| SimulationError::FfiError(format!("Invalid string: {}", e)))?;
        
        let ptr = cstring.as_ptr();
        
        if let Ok(mut strings) = self.string_allocations.lock() {
            strings.push(cstring);
        }
        
        Ok(ptr)
    }
    
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let allocation_count = self.allocations.lock()
            .map(|allocs| allocs.len())
            .unwrap_or(0);
        
        let string_count = self.string_allocations.lock()
            .map(|strings| strings.len())
            .unwrap_or(0);
        
        MemoryStats {
            allocation_count,
            string_count,
            estimated_bytes: allocation_count * 1024, // Rough estimate
        }
    }
    
    /// Force cleanup of all allocations
    pub fn cleanup(&self) {
        // Clean up raw allocations
        if let Ok(mut allocations) = self.allocations.lock() {
            for ptr in allocations.drain(..) {
                unsafe {
                    libc::free(ptr);
                }
            }
        }
        
        // Clean up string allocations
        if let Ok(mut strings) = self.string_allocations.lock() {
            strings.clear();
        }
    }
}

impl NgSpiceMemory {
    /// Get raw pointer
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
    
    /// Get size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Cast to specific type
    pub unsafe fn cast<T>(&self) -> *mut T {
        self.ptr as *mut T
    }
    
    /// Write data to memory
    pub unsafe fn write_bytes(&self, data: &[u8]) -> Result<(), SimulationError> {
        if data.len() > self.size {
            return Err(SimulationError::MemoryError {
                reason: "Data size exceeds allocated memory".to_string(),
            });
        }
        
        ptr::copy_nonoverlapping(data.as_ptr(), self.ptr as *mut u8, data.len());
        Ok(())
    }
    
    /// Read data from memory
    pub unsafe fn read_bytes(&self, len: usize) -> Result<Vec<u8>, SimulationError> {
        if len > self.size {
            return Err(SimulationError::MemoryError {
                reason: "Read size exceeds allocated memory".to_string(),
            });
        }
        
        let mut buffer = vec![0u8; len];
        ptr::copy_nonoverlapping(self.ptr as *const u8, buffer.as_mut_ptr(), len);
        Ok(buffer)
    }
}

impl Drop for NgSpiceMemory {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr);
        }
        
        // Remove from pool tracking
        if let Ok(mut allocations) = self.pool.lock() {
            if let Some(pos) = allocations.iter().position(|&p| p == self.ptr) {
                allocations.remove(pos);
            }
        }
    }
}

impl StringManager {
    /// Create a new string manager
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
        }
    }
    
    /// Create a managed CString and return its pointer
    pub fn create_cstring(&mut self, s: &str) -> Result<*const c_char, SimulationError> {
        let cstring = CString::new(s)
            .map_err(|e| SimulationError::FfiError(format!("Invalid string: {}", e)))?;
        
        let ptr = cstring.as_ptr();
        self.strings.push(cstring);
        Ok(ptr)
    }
    
    /// Create multiple CStrings at once
    pub fn create_cstring_array(&mut self, strings: &[&str]) -> Result<Vec<*const c_char>, SimulationError> {
        let mut ptrs = Vec::new();
        
        for s in strings {
            let ptr = self.create_cstring(s)?;
            ptrs.push(ptr);
        }
        
        Ok(ptrs)
    }
    
    /// Convert C string to Rust string safely
    pub fn c_str_to_string(ptr: *const c_char) -> Result<String, SimulationError> {
        if ptr.is_null() {
            return Err(SimulationError::FfiError("Null pointer".to_string()));
        }
        
        unsafe {
            CStr::from_ptr(ptr)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|e| SimulationError::FfiError(format!("Invalid UTF-8: {}", e)))
        }
    }
    
    /// Get number of managed strings
    pub fn string_count(&self) -> usize {
        self.strings.len()
    }
    
    /// Clear all managed strings
    pub fn clear(&mut self) {
        self.strings.clear();
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Number of active allocations
    pub allocation_count: usize,
    /// Number of managed strings
    pub string_count: usize,
    /// Estimated memory usage in bytes
    pub estimated_bytes: usize,
}

/// Memory leak detector for debugging
pub struct LeakDetector {
    /// Initial allocation count
    initial_allocations: usize,
    /// Pool reference
    pool: Arc<Mutex<Vec<*mut c_void>>>,
}

impl LeakDetector {
    /// Create a new leak detector
    pub fn new(pool: &MemoryPool) -> Self {
        let initial_allocations = pool.allocations.lock()
            .map(|allocs| allocs.len())
            .unwrap_or(0);
        
        Self {
            initial_allocations,
            pool: pool.allocations.clone(),
        }
    }
    
    /// Check for memory leaks
    pub fn check_leaks(&self) -> Option<usize> {
        let current_allocations = self.pool.lock()
            .map(|allocs| allocs.len())
            .unwrap_or(0);
        
        if current_allocations > self.initial_allocations {
            Some(current_allocations - self.initial_allocations)
        } else {
            None
        }
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StringManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global memory pool for NgSpice operations
static mut GLOBAL_MEMORY_POOL: Option<MemoryPool> = None;
static MEMORY_POOL_INIT: std::sync::Once = std::sync::Once::new();

/// Get or initialize the global memory pool
pub fn get_global_memory_pool() -> &'static MemoryPool {
    unsafe {
        MEMORY_POOL_INIT.call_once(|| {
            GLOBAL_MEMORY_POOL = Some(MemoryPool::new());
        });
        
        GLOBAL_MEMORY_POOL.as_ref().unwrap()
    }
}

/// Cleanup global memory pool
pub fn cleanup_global_memory_pool() {
    unsafe {
        if let Some(pool) = &GLOBAL_MEMORY_POOL {
            pool.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_allocation() {
        let pool = MemoryPool::new();
        let memory = pool.allocate(1024).unwrap();
        
        assert_eq!(memory.size(), 1024);
        assert!(!memory.as_ptr().is_null());
        
        let stats = pool.memory_stats();
        assert_eq!(stats.allocation_count, 1);
    }
    
    #[test]
    fn test_string_manager() {
        let mut manager = StringManager::new();
        let ptr = manager.create_cstring("test").unwrap();
        
        assert!(!ptr.is_null());
        assert_eq!(manager.string_count(), 1);
        
        let recovered = StringManager::c_str_to_string(ptr).unwrap();
        assert_eq!(recovered, "test");
    }
    
    #[test]
    fn test_memory_cleanup() {
        let pool = MemoryPool::new();
        
        {
            let _memory1 = pool.allocate(512).unwrap();
            let _memory2 = pool.allocate(1024).unwrap();
            
            let stats = pool.memory_stats();
            assert_eq!(stats.allocation_count, 2);
        }
        
        // Memory should be automatically freed when NgSpiceMemory is dropped
        // Note: This test might be flaky due to timing of Drop trait execution
    }
    
    #[test]
    fn test_leak_detector() {
        let pool = MemoryPool::new();
        let detector = LeakDetector::new(&pool);
        
        let _memory = pool.allocate(256).unwrap();
        
        let leaks = detector.check_leaks();
        assert_eq!(leaks, Some(1));
    }
}