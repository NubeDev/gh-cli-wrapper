use std::ffi::OsString;

pub(crate) fn push_opt(args: &mut Vec<OsString>, flag: &str, value: &Option<String>) {
    if let Some(value) = value {
        args.push(flag.into());
        args.push(value.into());
    }
}

pub(crate) fn push_opt_u32(args: &mut Vec<OsString>, flag: &str, value: Option<u32>) {
    if let Some(value) = value {
        args.push(flag.into());
        args.push(value.to_string().into());
    }
}

pub(crate) fn push_repeated(args: &mut Vec<OsString>, flag: &str, values: &[String]) {
    for value in values {
        args.push(flag.into());
        args.push(value.into());
    }
}

pub(crate) fn push_bool(args: &mut Vec<OsString>, flag: &str, value: bool) {
    if value {
        args.push(flag.into());
    }
}
