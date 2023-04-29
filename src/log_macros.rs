#[macro_export]
macro_rules! plain {
  ($log: expr, $fmt: expr) => {
    $log.plain(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.plain(format_args!($fmt, $($args)+))
  };
}

#[macro_export]
macro_rules! info {
  ($log: expr, $fmt: expr) => {
    $log.info(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.info(format_args!($fmt, $($args)+))
  };
}

#[macro_export]
macro_rules! warning {
  ($log: expr, $fmt: expr) => {
    $log.warning(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.warning(format_args!($fmt, $($args)+))
  };
}

#[macro_export]
macro_rules! error {
  ($log: expr, $fmt: expr) => {
    $log.error(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.error(format_args!($fmt, $($args)+))
  };
}
