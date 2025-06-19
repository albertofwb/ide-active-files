use crate::types::{ProcessInfo, SupportedIDE};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("No {ide} processes found")]
    NoProcessFound { ide: String },
    #[error("Failed to parse window information: {message}")]
    WindowParseError { message: String },
    #[error("System API error: {message}")]
    SystemError { message: String },
    #[error("Unsupported IDE: {ide}")]
    UnsupportedIDE { ide: String },
}

pub type DetectionResult<T> = Result<T, DetectionError>;

/// IDE detection strategy trait
pub trait IDEDetector {
    /// Get IDE type
    fn ide_type(&self) -> SupportedIDE;
    
    /// Check if process is target IDE
    fn is_target_process(&self, process: &ProcessInfo) -> bool;
    
    /// Extract file information from processes
    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<crate::types::DetectionResult>;
    
    /// Get IDE display name
    fn display_name(&self) -> &'static str {
        self.ide_type().display_name()
    }
}

/// IDE detector manager
pub struct IDEDetectorManager {
    detectors: Vec<Box<dyn IDEDetector>>,
}

impl IDEDetectorManager {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn register_detector(&mut self, detector: Box<dyn IDEDetector>) {
        self.detectors.push(detector);
    }

    pub fn detect_ide(&self, ide_type: SupportedIDE) -> DetectionResult<crate::types::DetectionResult> {
        let detector = self.detectors.iter()
            .find(|d| d.ide_type() == ide_type)
            .ok_or_else(|| DetectionError::UnsupportedIDE { 
                ide: ide_type.display_name().to_string() 
            })?;

        let processes = crate::process::find_all_processes()?;
        let target_processes: Vec<_> = processes.iter()
            .filter(|p| detector.is_target_process(p))
            .cloned()
            .collect();

        if target_processes.is_empty() {
            return Err(DetectionError::NoProcessFound { 
                ide: detector.display_name().to_string() 
            });
        }

        detector.extract_files(&target_processes)
    }

    pub fn auto_detect(&self) -> DetectionResult<crate::types::DetectionResult> {
        let processes = crate::process::find_all_processes()?;
        
        for detector in &self.detectors {
            let target_processes: Vec<_> = processes.iter()
                .filter(|p| detector.is_target_process(p))
                .cloned()
                .collect();

            if !target_processes.is_empty() {
                return detector.extract_files(&target_processes);
            }
        }

        Err(DetectionError::NoProcessFound { 
            ide: "any supported IDE".to_string() 
        })
    }

    pub fn list_supported_ides(&self) -> Vec<&'static str> {
        self.detectors.iter()
            .map(|d| d.display_name())
            .collect()
    }
}