# Change Log

## 0.0.3

### Features

- Add `with_executor` in ReplayOptions.
- Add `PacketFeedback` in feedbacks.

## 0.0.2 - 2025-12-26

### Features

- Add `RustlsClient` with NoVerify in Tavern. To enable it, add "rustls" feature.
- Add `RandExt` in Common, user can generate random bytes.
- Add `PacketObserver` in observers.

## 0.0.1 - 2025-09-25

### Features

- Some CLI tools for fuzzing.
    - `BaseFuzzerOptions` has some basic options.
    - `ReplayOptions` has some replay options.
- Traditional memory tools.
    - `has_new_bits` is a classical way to compare memory regions.
    - `get_layout` will allocate some memory and get its layout.
    - `read_from_mem`, `write_to_mem` read and write from/to `UnixShMem`.
    - `read_from_ptr`, `write_to_ptr` read and write from/to raw u8 ptr.
    - `show_shm_mem_layout`, `show_shm_ptr_layout` will print memory layout.
- Process tools.
    - `pidof` will get the pid of a process by name.
    - `is_proc_alive` will check if a process is alive.
- Tavern has any kind of harnesses for different targets.
    - Harness based on OpenSSL.
    - Harness based on TCP/UDP.
