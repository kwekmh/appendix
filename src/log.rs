use std::fmt;
use std::sync::{Arc, RwLock};

pub struct Record {
    value: Box<Vec<u8>>,
    offset: usize
}

#[derive(Clone)]
pub struct Log {
    records: Arc<RwLock<Vec<Arc<Record>>>>
}

#[derive(Debug, Clone)]
pub struct RecordNotFoundError {
    pub message: String,
}

impl Record {
    pub fn new(data: Vec<u8>) -> Record {
        Record {
            value: Box::new(data),
            offset: 0
        }
    }

    pub fn value(&self) -> Box<Vec<u8>> {
        self.value.clone()
    }
}

impl Log {
    pub fn new() -> Log {
        Log {
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn append(&mut self, mut record: Record) -> usize {
        let mut data = self.records.write().unwrap();
        let offset = data.len();
        record.offset = offset;
        let record_new = Arc::new(record);
        data.push(record_new);
        drop(data);
        offset
    }

    pub fn read(&self, offset: usize) -> Result<Arc<Record>, RecordNotFoundError> {
        let data = self.records.read().unwrap();
        if offset <= data.len() {
            Ok(data[offset].clone())
        } else {
            Err(RecordNotFoundError {
                message: format!("Offset {} not found", offset).to_string()
            })
        }
    }
}

impl fmt::Display for RecordNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}