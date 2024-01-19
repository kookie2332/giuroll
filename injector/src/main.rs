#![feature(core_intrinsics)]
use std::time::Duration;
use std::env::current_dir;
use dll_syringe::{process::OwnedProcess, Syringe};
use dll_syringe::process::{BorrowedProcessModule};
use dll_syringe::rpc::RemoteRawProcedure;

// TODO: REMOVE THIS BEFORE PR.
fn print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}

fn get_better_exe_init(syringe: &Syringe, injected_payload: BorrowedProcessModule<'_>)
    -> Option<RemoteRawProcedure<extern "C" fn() -> bool>> {
    // Try getting the better_exe_init procedure from giuroll.dll.
    let init_fn_name = "better_exe_init";
    let remote_better_exe_init_process = unsafe {
        syringe.get_raw_procedure::<extern "C" fn() -> bool>(injected_payload, init_fn_name)
    };

    let remote_better_exe_init_result = remote_better_exe_init_process.unwrap();

    remote_better_exe_init_result
}

fn get_better_push_path(syringe: &Syringe, injected_payload: BorrowedProcessModule<'_>)
    -> Option<RemoteRawProcedure<extern "C" fn(u8)>> {
    // Try getting the push_path procedure from giuroll.dll.
    let push_path_fn_name = "better_exe_init_push_path";
    let remote_push_path_process = unsafe {
        syringe.get_raw_procedure::<extern "C" fn(u8)>(injected_payload, push_path_fn_name)
    };

    let remote_push_path_result = remote_push_path_process.unwrap();

    remote_push_path_result
}

fn main() {
    // Check if th123.exe is running.
    let target_process = match OwnedProcess::find_first_by_name("th123.exe") {
        Some(x) => x,
        None => {
            println!("th123.exe process not found, make sure soku is running");
            std::thread::sleep(Duration::from_secs(5));
            panic!()
        }
    };

    // Inject giuroll.dll into the game.
    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe.inject("giuroll.dll").unwrap();

    //print_type(&remote_better_exe_init_result);
    // "Option<RemoteRawProcedure<extern \"C\" fn() -> bool>>"

    // put the exe_init together with the push_path method and the current directory.
    // The idea is that if just one of these gives None, the remaining map/and_then instructions will not run, saving
    // runtime.
    let mapped = get_better_exe_init(&syringe, injected_payload)
        .map(|f1| (f1, get_better_push_path(&syringe, injected_payload).unwrap()))
        .and_then(|(f1, f2)| current_dir().ok().map(move |path| (f1, f2, path)));
    // if fail anywhere, just load default exe

    match mapped {
        Some((f1, f2, current_path)) => {
            // We meed f1, f2 and the current path to be successfully extracted.
            let slice = current_path.as_os_str().as_encoded_bytes();
            for a in slice {
                f2.call(*a).unwrap(); // push path
            }
            if f1.call().unwrap() {
                println!("injection successful.")
            } else {
                println!("injection failed, giuroll.ini not found.")
            }
        }
        None => unsafe {
            // Try and use the exeinit method instead.
            syringe.get_raw_procedure::<unsafe extern "C" fn()>(injected_payload, "exeinit")
            .unwrap()
            .unwrap()
            .call()
            .unwrap()
        }
    };
    std::thread::sleep(Duration::from_secs(5));
}
