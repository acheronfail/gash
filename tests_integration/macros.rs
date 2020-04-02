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

#[macro_export]
macro_rules! eqnice {
  ($expected:expr, $got:expr) => {
    let expected = &*$expected;
    let got = &*$got;
    if expected != got {
      panic!(
        "
printed outputs differ!

expected:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
{}
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

got:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
{}
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
",
        expected, got
      );
    }
  };
}
