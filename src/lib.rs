use std::sync::mpsc::Sender;

use pyo3::prelude::*;

/// This Python class holds a [`Sender`]. This way, when the Python interpreter drops it
/// during interpreter deinitialization, the [`Sender`] being dropped is able to kill 
/// another thread.
#[pyclass]
pub struct SenderHandleHolder(Sender<()>);

/// Spawn a thread from within Rust, and return a Python object that kills the thread 
/// when dropped.
/// 
/// On exit, the thread will try to acquire the GIL, but this will cause a panic.
#[pyfunction]
fn spawn_thread_and_get_kill_handle() -> SenderHandleHolder {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || { 
        let _ = rx.recv();
        println!("(message through rust) the thread is exiting");
        Python::with_gil(|py| {
            let builtins = PyModule::import(py, "builtins")?;
            builtins.getattr("print")?.call1(("(message through python) the thread is exiting", ))?;

            Ok::<(), PyErr>(())
        }).expect("couldn't find or call the print function");
    });
    SenderHandleHolder(tx)
}


#[pymodule]
fn with_gil_panic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(spawn_thread_and_get_kill_handle, m)?)?;
    m.add_class::<SenderHandleHolder>()?;
    Ok(())
}