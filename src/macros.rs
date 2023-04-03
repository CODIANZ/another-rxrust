#[macro_export]
macro_rules! junk_next (
  () => ( |_| {} )
);

#[macro_export]
macro_rules! junk_error (
  () => ( |_| {} )
);

#[macro_export]
macro_rules! junk_complete (
  () => ( || {} )
);

#[macro_export]
macro_rules! panic_error (
  () => ( |e| { panic!("{:?}", e.any_ref()); } )
);

#[macro_export]
macro_rules! print_next (
  () => ( |_| { println!("next"); } )
);

#[macro_export]
macro_rules! print_next_fmt (
  ($e: expr) => ( |x| { println!("next - {}", format!($e, x)); } )
);

#[macro_export]
macro_rules! print_error (
  () => ( |e| { println!("error - {:?}", e.any_ref()); } )
);

#[macro_export]
macro_rules! print_complete (
  () => ( || { println!("complete"); } )
);
