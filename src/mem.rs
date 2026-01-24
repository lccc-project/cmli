use core::mem::ManuallyDrop;

pub const unsafe fn transmute_unchecked<T, U>(x: T) -> U {
    union Transmuter<T, U> {
        src: ManuallyDrop<T>,
        dest: ManuallyDrop<U>,
    }

    ManuallyDrop::into_inner(unsafe {
        Transmuter {
            src: ManuallyDrop::new(x),
        }
        .dest
    })
}

pub const unsafe fn transmute<T, U>(x: T) -> U {
    const {
        assert!(size_of::<T>() == size_of::<U>());
    }
    unsafe { transmute_unchecked::<T, U>(x) }
}
