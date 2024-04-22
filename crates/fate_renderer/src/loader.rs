use rendering::model::{Model, ModelStagingResources};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use vulkan::ash::vk;
use vulkan::{Context, PreLoadedResource};

pub struct Loader {
    message_sender: Sender<Message>,
    model_receiver: Receiver<PreLoadedResource<Model, ModelStagingResources>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Loader {
    pub fn new(context: Arc<Context>) -> Self {
        let (message_sender, message_receiver) = mpsc::channel();
        let (model_sender, model_receiver) = mpsc::channel();

        let thread_handle = Some(thread::spawn(move || loop {
            let message = message_receiver.recv().expect("接收路径错误！");
            match message {
                Message::Load(path) => {
                    log::info!("{}加载中...", path.as_path().display());
                    let pre_loaded_model = pre_load_model(&context, path.as_path());

                    match pre_loaded_model {
                        Ok(pre_loaded_model) => {
                            log::info!("{}加载成功", path.as_path().display());
                            model_sender.send(pre_loaded_model).unwrap();
                        }
                        Err(error) => {
                            log::error!("{}载入失败，由于:{}", path.as_path().display(), error);
                        }
                    }
                }
                Message::Stop => break,
            }
        }));

        Self {
            message_sender,
            model_receiver,
            thread_handle,
        }
    }

    pub fn load(&self, path: PathBuf) {
        self.message_sender
            .send(Message::Load(path))
            .expect("路径发送错误！");
    }

    pub fn get_model(&self) -> Option<Model> {
        match self.model_receiver.try_recv() {
            Ok(mut pre_loaded_model) => Some(pre_loaded_model.finish()),
            _ => None,
        }
    }
}

fn pre_load_model<P: AsRef<Path>>(
    context: &Arc<Context>,
    path: P,
) -> Result<PreLoadedResource<Model, ModelStagingResources>, Box<dyn Error>> {
    let device = context.device();

    let command_buffer = {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(context.general_command_pool())
            .level(vk::CommandBufferLevel::SECONDARY)
            .command_buffer_count(1);

        unsafe { device.allocate_command_buffers(&allocate_info).unwrap()[0] }
    };

    {
        let inheritance_info = vk::CommandBufferInheritanceInfo::builder().build();
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .inheritance_info(&inheritance_info)
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .unwrap()
        };
    }

    let model = Model::create_from_file(Arc::clone(context), command_buffer, path);

    unsafe { device.end_command_buffer(command_buffer).unwrap() };

    model
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.message_sender
            .send(Message::Stop)
            .expect("发送停止消息错误！");
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("无法等待加载线程终止！");
        }
        log::info!("卸载加载器");
    }
}

enum Message {
    Load(PathBuf),
    Stop,
}
