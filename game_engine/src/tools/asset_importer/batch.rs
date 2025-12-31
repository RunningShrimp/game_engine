//! # 批量导入（Batch Importer）
//!
//! 支持批量导入多个资源文件，带进度跟踪。

use crate::tools::asset_importer::importer::{AssetImporter, ImportResult, ImportError};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 批量导入器
pub struct BatchImporter {
    files: Vec<PathBuf>,
    settings: BatchImportSettings,
    progress: Arc<Mutex<BatchProgress>>,
}

impl BatchImporter {
    /// 创建新的批量导入器
    pub fn new(files: Vec<PathBuf>, output_directory: PathBuf) -> Self {
        Self {
            files,
            settings: BatchImportSettings {
                output_directory,
                continue_on_error: true,
                parallel: false,
                max_parallel: 4,
            },
            progress: Arc::new(Mutex::new(BatchProgress {
                total: 0,
                completed: 0,
                failed: 0,
                current_file: None,
            })),
        }
    }

    /// 设置批量导入选项
    pub fn with_settings(mut self, settings: BatchImportSettings) -> Self {
        self.settings = settings;
        self
    }

    /// 获取进度
    pub async fn get_progress(&self) -> BatchProgress {
        self.progress.lock().await.clone()
    }

    /// 导入所有文件
    pub async fn import_all(&mut self) -> Result<BatchReport, ImportError> {
        let mut results = Vec::new();
        let total = self.files.len();

        // 更新进度
        {
            let mut progress = self.progress.lock().await;
            progress.total = total;
            progress.completed = 0;
            progress.failed = 0;
        }

        // 创建导入器
        let importer = AssetImporter::new(self.settings.output_directory.clone());

        // 串行导入
        for file in &self.files {
            // 更新当前文件
            {
                let mut progress = self.progress.lock().await;
                progress.current_file = Some(file.clone());
            }

            match importer.import(file) {
                Ok(result) => {
                    results.push(Ok(result));
                    {
                        let mut progress = self.progress.lock().await;
                        progress.completed += 1;
                    }
                }
                Err(e) => {
                    results.push(Err(e));
                    {
                        let mut progress = self.progress.lock().await;
                        progress.failed += 1;
                    }

                    if !self.settings.continue_on_error {
                        break;
                    }
                }
            }
        }

        // 清除当前文件
        {
            let mut progress = self.progress.lock().await;
            progress.current_file = None;
        }

        Ok(BatchReport {
            total_files: total,
            successful_imports: results.iter().filter(|r| r.is_ok()).count(),
            failed_imports: results.iter().filter(|r| r.is_error()).count(),
            results,
        })
    }

    /// 并行导入所有文件
    pub async fn import_all_parallel(&mut self) -> Result<BatchReport, ImportError> {
        let mut results = Vec::new();
        let total = self.files.len();

        // 更新进度
        {
            let mut progress = self.progress.lock().await;
            progress.total = total;
            progress.completed = 0;
            progress.failed = 0;
        }

        // 使用信号量限制并发数
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.settings.max_parallel));
        let mut tasks = Vec::new();

        for file in self.files.clone() {
            let permit = semaphore.clone();
            let output_dir = self.settings.output_directory.clone();
            let progress = self.progress.clone();

            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();

                // 更新当前文件
                {
                    let mut prog = progress.lock().await;
                    prog.current_file = Some(file.clone());
                }

                let importer = AssetImporter::new(output_dir);
                let result = importer.import(&file);

                // 更新进度
                {
                    let mut prog = progress.lock().await;
                    if result.is_ok() {
                        prog.completed += 1;
                    } else {
                        prog.failed += 1;
                    }
                    prog.current_file = None;
                }

                result
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        for task in tasks {
            results.push(task.await.unwrap());
        }

        Ok(BatchReport {
            total_files: total,
            successful_imports: results.iter().filter(|r| r.is_ok()).count(),
            failed_imports: results.iter().filter(|r| r.is_error()).count(),
            results,
        })
    }
}

/// 批量导入设置
#[derive(Clone, Debug)]
pub struct BatchImportSettings {
    pub output_directory: PathBuf,
    pub continue_on_error: bool,
    pub parallel: bool,
    pub max_parallel: usize,
}

/// 批量进度
#[derive(Clone, Debug)]
pub struct BatchProgress {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub current_file: Option<PathBuf>,
}

/// 批量导入报告
#[derive(Clone, Debug)]
pub struct BatchReport {
    pub total_files: usize,
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub results: Vec<Result<ImportResult, ImportError>>,
}

// 辅助trait
trait IsError {
    fn is_error(&self) -> bool;
}

impl<T, E> IsError for Result<T, E> {
    fn is_error(&self) -> bool {
        self.is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_batch_import() {
        // 创建测试文件
        let test_dir = "/tmp/test_batch_import";
        fs::create_dir_all(test_dir).ok();

        let file1 = format!("{}/test1.png", test_dir);
        let img = image::RgbImage::new(50, 50);
        img.save(&file1).unwrap();

        let file2 = format!("{}/test2.png", test_dir);
        let img2 = image::RgbImage::new(100, 100);
        img2.save(&file2).unwrap();

        let files = vec![PathBuf::from(&file1), PathBuf::from(&file2)];

        let output_dir = "/tmp/test_batch_output";
        fs::create_dir_all(output_dir).ok();

        let mut batch = BatchImporter::new(files, PathBuf::from(output_dir));
        let report = batch.import_all().await.unwrap();

        assert_eq!(report.total_files, 2);
        assert_eq!(report.successful_imports, 2);

        // 清理
        fs::remove_dir_all(test_dir).ok();
        fs::remove_dir_all(output_dir).ok();
    }

    #[tokio::test]
    async fn test_batch_progress() {
        let files = vec![];
        let output_dir = "/tmp/test_batch_progress";

        let batch = BatchImporter::new(files, PathBuf::from(output_dir));
        let progress = batch.get_progress().await;

        assert_eq!(progress.total, 0);
        assert_eq!(progress.completed, 0);
        assert_eq!(progress.failed, 0);
    }
}
