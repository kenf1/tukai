use std::collections::HashMap;
use std::{error, io};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::file_handler::FileHandler;

use super::stats::{Stat, TypingDuration};
use super::activities::Activities;

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq, Debug)]
pub enum StorageDataType {
  Stats,
  Activities
}

#[derive(Deserialize, Serialize, Debug)]
pub enum StorageDataValue {
  Stats(Vec<Stat>),
  Activites(Activities)
}

impl StorageDataValue {
  // pub fn insert_stats(stat_name: String, stat_value: i32) -> Self {
  //   StorageDataValue::Stats(Stats { stat_name, stat_value })
  // }
}

type StorageData = HashMap<StorageDataType, StorageDataValue>;

pub struct StorageHandler {
  file_path: PathBuf,
  data: StorageData
}

impl StorageHandler {

  pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
    Self {
      file_path: file_path.as_ref().to_owned(),
      data: HashMap::new()
    }
  }

  /// Default data for the storage
  ///
  /// Create an empty Vec for stats
  /// Create an empty Vec for activities
  ///
  /// Store into a HashMap
  ///
  /// Writes into the binary file
  pub fn default(self) -> Result<Self, std::io::Error> {
    let mut empty_data: StorageData = HashMap::new();

    let empty_stats = StorageDataValue::Stats(Vec::new());
    let empty_activities= StorageDataValue::Activites(Vec::new());

    empty_data.insert(StorageDataType::Stats, empty_stats);
    empty_data.insert(StorageDataType::Activities, empty_activities);

    let data_bytes = bincode::serialize(&empty_data).unwrap();
    FileHandler::write_bytes_into_file(&self.file_path, &data_bytes)?;

    Ok(self)
  }

  /// Inits the storage
  ///
  /// Try to read all bytes from the storage file
  /// Then set into the data
  pub fn init(mut self) -> Result<Self, io::Error> {
    if let Ok(data_bytes) = FileHandler::read_bytes_from_file(&self.file_path) {
      self.data = bincode::deserialize(&data_bytes).unwrap();
    }

    Ok(self)
  }

  pub fn get_data(&self) -> &StorageData {
    &self.data
  }

  pub fn get_data_stats(&self) -> Option<&Vec<Stat>> {
    if let Some(StorageDataValue::Stats(stats)) = self.data.get(&StorageDataType::Stats) {
      Some(stats)
    } else {
      None
    }
  }

  pub fn get_data_stats_reversed(&self) -> Option<Vec<Stat>> {
    if let Some(StorageDataValue::Stats(stats)) = self.data.get(&StorageDataType::Stats) {
      let stats_reversed = stats.iter()
        .rev()
        .map(|item| item.to_owned()).collect::<Vec<Stat>>();

      Some(stats_reversed)
    } else {
      None
    }
  }

  /// Gets the stats from the storage
  fn get_data_stats_mut(&mut self) -> Option<&mut Vec<Stat>> {
    if let Some(StorageDataValue::Stats(stats)) = self.data.get_mut(&StorageDataType::Stats) {
      Some(stats)
    } else {
      None
    }
  }

  /// Gets the activities from the storage
  fn get_data_activities_mut(&mut self) -> Option<&Activities> {
    if let Some(StorageDataValue::Activites(activities)) = self.data.get_mut(&StorageDataType::Activities) {
      Some(activities)
    } else {
      None
    }
  }

  /// Loads all data
  fn load(&self) -> StorageData {
    let data_bytes = FileHandler::read_bytes_from_file(&self.file_path)
      .unwrap();

    let data = bincode::deserialize::<StorageData>(&data_bytes)
      .unwrap();

    data
  }

  /// Flush all data
  fn flush(&self) -> Result<(), std::io::Error> {
    let data_bytes = bincode::serialize(&self.data)
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    FileHandler::write_bytes_into_file("test.tukai", &data_bytes)
  }

  pub fn insert_into_stats(
    &mut self,
    stat: &Stat
  ) -> bool {
    if let Some(stats) = self.get_data_stats_mut() {
      stats.push(stat.clone());
      return self.flush().is_ok();
    }

    false
  }

}

#[cfg(test)]
mod tests {
  use crate::storage::stats::TypingDuration;
  use super::*;

  fn get_storage_handler() -> StorageHandler {
    let storage_helper = StorageHandler::new("test.tukai")
      .default()
      .unwrap()
      .init()
      .unwrap();

    storage_helper
  }

  fn get_test_stat() -> Stat {
    Stat::new(
      TypingDuration::Minute,
      80,
      5,
      60
    )
  }

  #[test]
  // Just validate if binary file was created right
  fn storage_load() {
    let storage_handler = get_storage_handler();
    let storage_data= storage_handler.load();

    assert!(storage_data.get(&StorageDataType::Stats).is_some(), "Stats not initialized successfully");
    assert!(storage_data.get(&StorageDataType::Activities).is_some(), "Activities not initialized successfully");
  }

  #[test]
  // Init an empty storage data
  //
  // Insert test Stat into the file
  //
  // Try to reverse read from the memory
  fn storage_insert_into_data_stats() {
    let mut storage_handler = get_storage_handler();

    let stat = get_test_stat();

    assert!(storage_handler.insert_into_stats(&stat), "Insert into the storage error occured");

    let stats = storage_handler.get_data_stats_mut();

    assert!(stats.is_some(), "Failed to read from the storage stats (stats is None)");

    let stats_unwraped = stats.unwrap();

    let stat_from_memory = &stats_unwraped[0];

    assert_eq!(stat_from_memory.get_average_wpm(), stat.get_average_wpm());
  }

  #[test]
  fn flush_data() {
    let mut storage_handler = get_storage_handler();
    storage_handler.insert_into_stats(&get_test_stat());

    assert!(storage_handler.flush().is_ok());
  }

  #[test]
  fn load_flushed_data() {
    let mut storage_handler = get_storage_handler();
    storage_handler.insert_into_stats(&get_test_stat());

    println!("{:?}", storage_handler.get_data());

    let data = storage_handler.load();
    println!("{:?}", data);
  }
}
