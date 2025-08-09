#[macro_export]
macro_rules! cfg_android {
  ($($item:item)*) => {
      $(
          #[cfg(all(feature = "android"))]
          #[cfg_attr(docsrs, doc(cfg(feature = "android")))]
          $item
      )*
  }
}

#[macro_export]
macro_rules! cfg_ios {
  ($($item:item)*) => {
      $(
          #[cfg(all(feature = "ios"))]
          #[cfg_attr(docsrs, doc(cfg(feature = "ios")))]
          $item
      )*
  }
}
