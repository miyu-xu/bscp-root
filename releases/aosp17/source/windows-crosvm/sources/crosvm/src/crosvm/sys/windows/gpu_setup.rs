// Shared Windows GPU configuration; constructs existing objects without starting workers.
use anyhow::{Context, Result};
use base::{BlockingMode, Event, FramingMode, SendTube, StreamChannel, Tube};
use crosvm_cli::sys::windows::exit::{Exit, ExitContext};
use devices::virtio::vhost_user_backend::gpu::sys::windows::{GpuBackendConfig, GpuVmmConfig, InputEventBackendConfig, InputEventSplitConfig, InputEventVmmConfig, WindowProcedureThreadSplitConfig, WindowProcedureThreadVmmConfig};
use gpu_display::{EventDevice, WindowProcedureThreadBuilder};
use crate::crosvm::config::InputDeviceOption;
use crate::sys::windows::{get_gpu_product_configs, get_window_procedure_thread_product_configs};
use crate::Config;

pub(crate) fn platform_create_input_event_config(cfg: &Config, default_keyboard: bool) -> Result<InputEventSplitConfig> {
    let mut event_devices = vec![];
    let mut multi_touch_pipes = vec![];
    let mut mouse_pipes = vec![];
    let mut keyboard_pipes = vec![];

    for input in &cfg.virtio_input {
        match input {
            InputDeviceOption::MultiTouch { .. } => {
                let (event_device_pipe, virtio_input_pipe) =
                    StreamChannel::pair(BlockingMode::Nonblocking, FramingMode::Byte)
                        .exit_context(Exit::EventDeviceSetup, "failed to set up EventDevice")?;
                event_devices.push(EventDevice::touchscreen(event_device_pipe));
                multi_touch_pipes.push(virtio_input_pipe);
            }
            InputDeviceOption::Mouse { .. } => {
                let (event_device_pipe, virtio_input_pipe) =
                    StreamChannel::pair(BlockingMode::Nonblocking, FramingMode::Byte)
                        .exit_context(Exit::EventDeviceSetup, "failed to set up EventDevice")?;
                event_devices.push(EventDevice::mouse(event_device_pipe));
                mouse_pipes.push(virtio_input_pipe);
            }
            _ => {}
        }
    }

    // Preserve broker default; ordinary run adds only explicitly requested input.
    if default_keyboard || cfg.virtio_input.iter().any(|input| matches!(input, InputDeviceOption::Keyboard { .. })) {
        // One keyboard
        let (event_device_pipe, virtio_input_pipe) =
            StreamChannel::pair(BlockingMode::Nonblocking, FramingMode::Byte)
                .exit_context(Exit::EventDeviceSetup, "failed to set up EventDevice")?;
        event_devices.push(EventDevice::keyboard(event_device_pipe));
        keyboard_pipes.push(virtio_input_pipe);
    }

    Ok(InputEventSplitConfig {
        backend_config: Some(InputEventBackendConfig { event_devices }),
        vmm_config: InputEventVmmConfig {
            multi_touch_pipes,
            mouse_pipes,
            keyboard_pipes,
        },
    })
}

/// Create Window procedure thread configurations.
pub(crate) fn platform_create_window_procedure_thread_configs(
    cfg: &Config,
    mut wndproc_thread_builder: WindowProcedureThreadBuilder,
    main_alias_pid: u32,
    device_alias_pid: u32,
) -> Result<WindowProcedureThreadSplitConfig> {
    if let Some(params) = cfg.gpu_parameters.as_ref() {
        wndproc_thread_builder.set_max_num_windows(params.max_num_displays);
    }
    let product_config = get_window_procedure_thread_product_configs(
        cfg,
        &mut wndproc_thread_builder,
        main_alias_pid,
        device_alias_pid,
    )
    .context("create product window procedure thread configs")?;
    Ok(WindowProcedureThreadSplitConfig {
        wndproc_thread_builder: Some(wndproc_thread_builder),
        vmm_config: WindowProcedureThreadVmmConfig { product_config },
    })
}

/// Create backend and VMM configurations for the GPU device.
pub(crate) fn platform_create_gpu(
    cfg: &Config,
    main_pid: u32,
    exit_events: &mut Vec<Event>,
    exit_evt_wrtube: SendTube,
) -> Result<Option<(GpuBackendConfig, GpuVmmConfig)>> {
    if cfg.gpu_parameters.is_none() {
        return Ok(None);
    }
    let exit_event = Event::new().exit_context(Exit::CreateEvent, "failed to create exit event")?;
    exit_events.push(
        exit_event
            .try_clone()
            .exit_context(Exit::CloneEvent, "failed to clone event")?,
    );

    let (backend_config_product, vmm_config_product) =
        get_gpu_product_configs(cfg, main_pid)?;

    let (mut main_vhost_user_tube, mut device_host_user_tube) =
        Tube::pair().exit_context(Exit::CreateTube, "failed to create tube")?;
    // Start off the vhost-user tube assuming it is in the main process.
    main_vhost_user_tube.set_target_pid(main_pid);
    device_host_user_tube.set_target_pid(main_pid);

    let (mut gpu_control_host_tube, mut gpu_control_device_tube) =
        Tube::pair().exit_context(Exit::CreateTube, "failed to create tube")?;
    gpu_control_host_tube.set_target_pid(main_pid);
    gpu_control_device_tube.set_target_pid(main_pid);

    let backend_config = GpuBackendConfig {
        device_vhost_user_tube: Some(device_host_user_tube),
        exit_event,
        exit_evt_wrtube,
        gpu_control_device_tube,
        params: cfg
            .gpu_parameters
            .as_ref()
            .expect("missing GpuParameters in config")
            .clone(),
        product_config: backend_config_product,
    };

    let vmm_config = GpuVmmConfig {
        main_vhost_user_tube: Some(main_vhost_user_tube),
        gpu_control_host_tube: Some(gpu_control_host_tube),
        product_config: vmm_config_product,
    };

    Ok(Some((backend_config, vmm_config)))
}


/// Own the in-process backend until its exit event has been signalled and joined.
pub(crate) struct MainProcessGpu {
    worker: Option<base::WorkerThread<Result<()>>>,
}
impl MainProcessGpu {
    fn finish_inner(&mut self) -> Result<()> {
        match self.worker.take() {
            Some(worker) => worker.stop(),
            None => Ok(()),
        }
    }
    pub(crate) fn finish(mut self) -> Result<()> { self.finish_inner() }
}
impl Drop for MainProcessGpu {
    fn drop(&mut self) {
        if let Err(error) = self.finish_inner() {
            base::error!("GPU backend cleanup failed during unwind: {error:#}");
        }
    }
}

pub(crate) fn prepare_main_process_gpu(cfg: &mut Config, vm_event: &SendTube)
    -> Result<Option<MainProcessGpu>> {
    if cfg.gpu_parameters.is_none() { return Ok(None); }
    anyhow::ensure!(cfg.gpu_backend_config.is_none() && cfg.gpu_vmm_config.is_none()
        && cfg.input_event_split_config.is_none()
        && cfg.window_procedure_thread_split_config.is_none(),
        "ordinary run cannot replace existing split GPU configuration");
    let pid = std::process::id();
    let mut exit_events = Vec::new();
    let (mut backend, frontend) = platform_create_gpu(
        cfg, pid, &mut exit_events, vm_event.try_clone()?,
    )?.context("explicit GPU parameters produced no GPU configuration")?;
    let mut input = platform_create_input_event_config(cfg, false)?;
    let mut window = platform_create_window_procedure_thread_configs(
        cfg, gpu_display::WindowProcedureThread::builder(), pid, pid,
    )?;
    let window_thread = window.wndproc_thread_builder.take()
        .context("GPU window thread builder missing")?
        .start_thread().context("start in-process GPU window dispatcher")?;
    let event_devices = input.backend_config.take()
        .context("GPU input backend missing")?.event_devices;
    let report = vm_event.try_clone()?;
    let worker = base::WorkerThread::start("gpu_backend", move |stop_event| {
        backend.exit_event = stop_event;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            devices::virtio::vhost_user_backend::gpu::sys::windows::run_gpu_device_worker(
                backend, event_devices, window_thread,
            )
        })).unwrap_or_else(|_| Err(anyhow::anyhow!("GPU backend panicked")));
        // Upstream run_handler returns Ok on frontend close as well as stop-event.
        // Frontend teardown can precede run_config_inner returning; do not turn it
        // into a spurious failure. Real handler errors/panics still notify the VMM.
        if let Err(error) = &result {
            base::error!("GPU backend failed: {error:#}");
            if let Err(send_error) = report.send(&base::VmEventType::DeviceCrashed) {
                base::error!("GPU failure notification failed: {send_error}");
            }
        }
        result
    });
    cfg.gpu_vmm_config = Some(frontend);
    cfg.input_event_split_config = Some(input);
    cfg.window_procedure_thread_split_config = Some(window);
    Ok(Some(MainProcessGpu { worker: Some(worker) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_gpu_has_no_backend_or_input() {
        let mut cfg = Config::default();
        let (send, _receive) = Tube::directional_pair().unwrap();
        assert!(prepare_main_process_gpu(&mut cfg, &send).unwrap().is_none());
        assert!(cfg.gpu_vmm_config.is_none());
        assert!(cfg.input_event_split_config.is_none());
    }
    #[test]
    fn ordinary_input_does_not_add_broker_keyboard() {
        let cfg = Config::default();
        let input = platform_create_input_event_config(&cfg, false).unwrap();
        assert!(input.vmm_config.keyboard_pipes.is_empty());
        assert!(input.backend_config.unwrap().event_devices.is_empty());
        let broker = platform_create_input_event_config(&cfg, true).unwrap();
        assert_eq!(broker.vmm_config.keyboard_pipes.len(), 1);
        assert_eq!(broker.backend_config.unwrap().event_devices.len(), 1);
    }
    #[test]
    fn frontend_close_remains_normal_worker_termination() {
        let mut cfg = Config::default();
        cfg.gpu_parameters = Some(Default::default());
        let (send, _receive) = Tube::directional_pair().unwrap();
        let worker = prepare_main_process_gpu(&mut cfg, &send).unwrap().unwrap();
        drop(cfg.gpu_vmm_config.take());
        std::thread::sleep(std::time::Duration::from_millis(100));
        worker.finish().unwrap();
    }
    #[test]
    fn real_gpu_backend_stop_is_owned_and_joined() {
        let mut cfg = Config::default();
        cfg.gpu_parameters = Some(Default::default());
        let (send, _receive) = Tube::directional_pair().unwrap();
        let worker = prepare_main_process_gpu(&mut cfg, &send).unwrap().unwrap();
        assert!(cfg.gpu_vmm_config.is_some());
        assert!(cfg.gpu_backend_config.is_none());
        let input = cfg.input_event_split_config.as_ref().unwrap();
        assert!(input.backend_config.is_none());
        assert!(input.vmm_config.keyboard_pipes.is_empty());
        assert!(cfg.window_procedure_thread_split_config.as_ref().unwrap()
            .wndproc_thread_builder.is_none());
        worker.finish().unwrap();
    }
}
