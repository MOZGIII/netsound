#[macro_export]
macro_rules! match_channels_explicit {
    ($frame:ident => [$channels:ident] => [$($N:expr)*] => $body:expr) => {
        match $channels {
            $(
                $N => {
                    type $frame<S> = [S; $N];
                    $body
                }
            )*
            _ => panic!("unsupported amount of channels"),
        }
    };
}

/// Go from channels number to Frame form at runtime.
///
/// Example:
///
/// ```
/// let channels = 1;
///
/// let val = match_channels! {
///     F => [channels] => {
///         format!("{:?}", F::<f32>::EQUILIBRIUM)
///     }
/// };
///
/// assert_eq!(val, "[0.0]");
/// ```
#[macro_export]
macro_rules! match_channels {
    ($frame:ident => [$channels:ident] => $body:expr) => {
        crate::match_channels_explicit! { $frame => [$channels] => [
            1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26
            27 28 29 30 31 32
        ] => $body }
    };
}

#[cfg(test)]
mod tests {
    use dasp_frame::Frame;

    #[test]
    fn test_1() {
        let channels = 1;

        let val = match_channels! {
            F => [channels] => {
                format!("{:?}", F::<f32>::EQUILIBRIUM)
            }
        };

        assert_eq!(val, "[0.0]");
    }

    #[test]
    fn test_2() {
        let channels = 2;

        let val = match_channels! {
             F => [channels] => {
                format!("{:?}", F::<f32>::EQUILIBRIUM)
            }
        };

        assert_eq!(val, "[0.0, 0.0]");
    }

    #[test]
    fn test_3() {
        let channels = 3;

        let val = match_channels! {
             F => [channels] => {
                format!("{:?}", F::<i16>::EQUILIBRIUM)
            }
        };

        assert_eq!(val, "[0, 0, 0]");
    }

    mod generic {
        use dasp_sample::Sample;
        use std::marker::PhantomData;

        struct MyGenericType<S: Sample> {
            s: PhantomData<S>,
            channels: usize,
        }

        impl<S: Sample + std::fmt::Debug> MyGenericType<S> {
            pub fn new(channels: usize) -> Self {
                Self {
                    s: PhantomData,
                    channels,
                }
            }

            pub fn frame_string(&mut self) -> String {
                let channels = self.channels;
                match_channels! {
                    F => [channels] => {
                        use dasp_frame::Frame;
                        format!("{:?}", F::<S>::EQUILIBRIUM)
                    }
                }
            }
        }

        #[test]
        fn test_generic_1() {
            let mut s = MyGenericType::<f32>::new(1);
            assert_eq!(s.frame_string(), "[0.0]");
        }

        #[test]
        fn test_generic_2() {
            let mut s = MyGenericType::<f32>::new(2);
            assert_eq!(s.frame_string(), "[0.0, 0.0]");
        }

        #[test]
        fn test_generic_3() {
            let mut s = MyGenericType::<i8>::new(3);
            assert_eq!(s.frame_string(), "[0, 0, 0]");
        }
    }
}
