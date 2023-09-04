use lean_base::*;

#[no_mangle]
pub extern "C"
fn lean_test_list_sum(xs: TObjRef<List<usize>>) -> usize {
   let mut cnt = 0;
   let mut xs = xs.to_owned();
   while let List::Cons(x, tail) = xs.unpack() {
      cnt += x.unpack();
      xs = tail;
   }
   cnt
}
