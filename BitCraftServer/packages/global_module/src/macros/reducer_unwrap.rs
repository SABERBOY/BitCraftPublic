#[macro_export]
macro_rules! unwrap_or_err(
    ($e:expr, $($str:tt)+) => (
        match $e {
            Some(v) => v,
            None => {
                spacetimedb::log::error!($($str)+);
                return Err(format!($($str)+))
            }
        }
    );
);

#[macro_export]
macro_rules! unwrap_or_return(
    ($e:expr, $($str:tt)+) => (
        match $e {
            Some(v) => v,
            None => {
                spacetimedb::log::error!($($str)+);
                return;
            }
        }
    );
);

#[macro_export]
macro_rules! unwrap_or_continue(
    ($e:expr, $($str:tt)+) => (
        match $e {
            Some(v) => v,
            None => {
                spacetimedb::log::error!($($str)+);
                continue;
            }
        }
    );
);
