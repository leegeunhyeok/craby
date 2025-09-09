#[macro_export]
macro_rules! handle_panic {
  ($func:expr, $($args:expr),*) => {
      std::panic::catch_unwind(|| {
          $func($($args),*)
      }).map_err(|e| {
          if let Some(s) = e.downcast_ref::<String>() {
              s.clone()
          } else if let Some(s) = e.downcast_ref::<&str>() {
              s.to_string()
          } else {
              "Unknown panic occurred".to_string()
          }
      })
  };
}
