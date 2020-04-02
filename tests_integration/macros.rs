#[macro_export]
macro_rules! gashtest {
  ($name:ident, $func:expr) => {
    #[test]
    fn $name() {
      let cmd = crate::util::setup(stringify!($name));
      $func(cmd);
    }
  };
}
