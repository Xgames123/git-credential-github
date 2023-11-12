#[macro_export()]
macro_rules! die {
    ($($arg:tt)+) => {log::error!("FATAL: {}", format!($($arg)+)); std::process::exit(1);}
}
