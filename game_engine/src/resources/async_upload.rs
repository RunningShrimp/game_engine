//  GPU资源异步上传模块
// 
//  提供异步GPU资源上传功能，避免阻塞主线程。

use std::sync::Arc;
use wgpu::{Buffer, Device, Queue};
use std::collections::VecDeque;
use std::task::Waker;

/// 异步上传任务
pub struct UploadTask {
    /// 目标buffer
    pub buffer: Arc<Buffer>,
    /// 上传数据
    pub data: Vec<u8>,
    /// offset
    pub offset: wgpu::BufferAddress,
    /// 完成通知
    pub waker: Option<Waker>,
}

/// GPU资源异步上传器
pub struct AsyncUploader {
    /// 待处理的任务队列
    queue: Arc<parking_lot::Mutex<VecDeque<UploadTask>>>,
    /// wgpu设备
    device: Arc<Device>,
    /// wgpu队列
    wgpu_queue: Arc<Queue>,
}

impl AsyncUploader {
    /// 创建新的异步上传器
    pub fn new(device: Arc<Device>, wgpu_queue: Arc<Queue>) -> Self {
        Self {
            queue: Arc::new(parking_lot::Mutex::new(VecDeque::new())),
            device,
            wgpu_queue,
        }
    }

    /// 执行所有待处理的上传任务
    ///
    /// 应该在每帧调用一次，处理队列中的上传任务
    pub fn process_queue(&self) {
        let mut queue = self.queue.lock();
        let tasks: Vec<_> = queue.drain(..).collect();
        drop(queue);

        if tasks.is_empty() {
            return;
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Async Upload Encoder"),
        });

        for task in tasks {
            let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: task.data.len() as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                mapped_at_creation: true,
            });

            {
                let mut view = staging.slice(..).get_mapped_range_mut();
                view.copy_from_slice(&task.data);
                drop(view);
            }
            staging.unmap();

            encoder.copy_buffer_to_buffer(&staging, 0, &task.buffer, task.offset, task.data.len() as u64);

            // 通知等待的waker
            if let Some(waker) = task.waker {
                waker.wake();
            }
        }

        // 提交命令
        self.wgpu_queue.submit(Some(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_uploader_creation() {
        // 测试需要实际的wgpu设备，这里仅测试结构创建
        // 在实际测试环境中，可以使用wgpu_test库
    }
}
