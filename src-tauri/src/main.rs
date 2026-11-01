// Entry point delegates to the library so integration tests can reuse it.

fn main() {
    // WebKitGTK's DMA-BUF renderer causes protocol errors (Error 71) on Wayland
    // with NVIDIA/Nouveau drivers on physical hardware, as well as blank windows
    // on virtual GPUs. Force it disabled by default on all hardware.
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    soplos_welcome_live_lib::run();
}
