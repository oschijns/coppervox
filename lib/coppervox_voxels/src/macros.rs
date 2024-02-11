/// build an array of 3 elements
#[macro_export]
macro_rules! array3 {
    (for $index:ident in { $expression:expr } -> $type:ty ) => {
        {
            let mut __array = <[$type; 3]>::default();
            seq_macro::seq![$index in 0..3 {
                __array[$index] = ($expression) as $type;
            }];
            __array
        }
    };
}

/// build a tuple of 3 elements
#[macro_export]
macro_rules! tuple3 {
    (for $index:ident in { $expression:expr } -> $type:ty ) => {
        {
            let mut __tuple = <($type, $type, $type)>::default();
            seq_macro::seq![$index in 0..3 {
                __tuple.$index = ($expression) as $type;
            }];
            __tuple
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let a = [1i32, 2, 3];
        let b = (3, 4, 5);

        let _c = tuple3!(for N in { a[N] + b.N } -> usize);
        let _d = array3!(for N in { a[N] + b.N } -> u64);
    }
}
