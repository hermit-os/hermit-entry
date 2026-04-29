#![cfg(not(target_os = "none"))]

#[test]
fn test_parse_tar() {
    let mut tar_data = tar::Builder::new(Vec::new());

    // file: `/hermit.toml`
    {
        let config_data = r#"
version = "1"
kernel = "hkernel"

[input]
kernel_args = []
app_args = ["testname=hello world"]
env_vars = []
"#;

        let mut header = tar::Header::new_ustar();
        header.set_path("hermit.toml").unwrap();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(config_data.len().try_into().unwrap());
        header.set_cksum();
        tar_data.append(&header, config_data.as_bytes()).unwrap();
    }

    // file: `/hkernel`
    let kernel_data = "ELF\0\nmeow".as_bytes();
    {
        let mut header = tar::Header::new_ustar();
        header.set_path("hkernel").unwrap();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(kernel_data.len().try_into().unwrap());
        header.set_cksum();
        tar_data.append(&header, kernel_data).unwrap();
    }

    tar_data.finish().unwrap();

    let parsed = hermit_entry::config::parse_tar(&tar_data.get_ref())
        .expect("unable to parse ustar archive");

    match &parsed.config {
        hermit_entry::config::Config::V1 { input, .. } => {
            assert_eq!(&input.app_args[..], &["testname=hello world".to_string()]);
        }
    }

    assert_eq!(parsed.raw_kernel, kernel_data);
}
