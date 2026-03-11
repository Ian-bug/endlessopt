use sysinfo::System;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryStatus {
    pub memory_load: u32,
    pub total_phys: u64,
    pub avail_phys: u64,
    pub total_page_file: u64,
    pub avail_page_file: u64,
    pub total_virtual: u64,
    pub avail_virtual: u64,
    pub avail_extended_virtual: u64,
}

impl MemoryStatus {
    /// Get current memory status using sysinfo
    pub fn get() -> Result<Self, Box<dyn std::error::Error>> {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();
        let used_memory = total_memory - available_memory;
        let memory_load = if total_memory > 0 {
            ((used_memory as f64 / total_memory as f64) * 100.0) as u32
        } else {
            0
        };

        Ok(MemoryStatus {
            memory_load,
            total_phys: total_memory,
            avail_phys: available_memory,
            total_page_file: 0,
            avail_page_file: 0,
            total_virtual: sys.total_swap(),
            avail_virtual: sys.free_swap(),
            avail_extended_virtual: 0,
        })
    }

    /// Get available physical memory in bytes
    pub fn get_available_memory() -> Result<u64, Box<dyn std::error::Error>> {
        Ok(Self::get()?.avail_phys)
    }

    /// Get total physical memory in bytes
    pub fn get_total_memory() -> Result<u64, Box<dyn std::error::Error>> {
        Ok(Self::get()?.total_phys)
    }

    /// Get memory load percentage (0-100)
    pub fn get_memory_load_percent() -> Result<u32, Box<dyn std::error::Error>> {
        Ok(Self::get()?.memory_load)
    }

    /// Get memory usage percentage as f32
    pub fn get_memory_usage_percent() -> Result<f32, Box<dyn std::error::Error>> {
        let status = Self::get()?;
        Ok((status.memory_load as f32))
    }

    /// Get used physical memory in bytes
    pub fn get_used_memory() -> Result<u64, Box<dyn std::error::Error>> {
        let status = Self::get()?;
        Ok(status.total_phys - status.avail_phys)
    }

    /// Format bytes to human readable string
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_memory_status() {
        let status = MemoryStatus::get().unwrap();
        assert!(status.total_phys > 0);
        assert!(status.avail_phys > 0);
        assert!(status.memory_load <= 100);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(MemoryStatus::format_bytes(500), "500 B");
        assert!(MemoryStatus::format_bytes(2048).contains("KB"));
        assert!(MemoryStatus::format_bytes(1024 * 1024 * 2).contains("MB"));
        assert!(MemoryStatus::format_bytes(1024 * 1024 * 1024 * 2).contains("GB"));
    }
}
