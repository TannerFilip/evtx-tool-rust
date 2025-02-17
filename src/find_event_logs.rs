use std::path::PathBuf;
use std::fs;

pub fn find_event_logs(input_path: String) -> Vec<PathBuf> {
    fn evtx_matcher(buf: &[u8]) -> bool {
        buf.len() >= 7
            && buf[0] == 0x45
            && buf[1] == 0x6C
            && buf[2] == 0x66
            && buf[3] == 0x46
            && buf[4] == 0x69
            && buf[5] == 0x6C
            && buf[6] == 0x65
    }
    let mut evtx_list = Vec::new();
    let mut info = infer::Infer::new();
    info.add("application/x-evtx", "evtx", evtx_matcher);
    let input_path_buf = PathBuf::from(&input_path);
    let abs_path = input_path_buf.canonicalize().unwrap();

    let directory = fs::read_dir(abs_path).unwrap();
    for file in directory {
        match file {
            Ok(f) => {
                let current_file_name = f.path();
                if current_file_name.is_file() {
                    let file_infer = info.get_from_path(&current_file_name);
                    match file_infer {
                        Ok(Some(file_type)) => match file_type.mime_type() {
                            "application/x-evtx" => evtx_list.push(current_file_name),
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("{:?} is not a valid evtx file", &current_file_name);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
    println!("{:?}", &evtx_list);
    evtx_list
}