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
  () => ( |e| { panic!("{:?}", e.error); } )
);

#[macro_export]
macro_rules! print_next (
  () => ( |x| { println!("{:?}", x); } )
);

#[macro_export]
macro_rules! print_error (
  () => ( |e| { println!("{:?}", e.error); } )
);

#[macro_export]
macro_rules! print_complete (
  () => ( || { println!("complete"); } )
);
