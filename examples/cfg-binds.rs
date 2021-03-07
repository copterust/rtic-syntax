//! Full syntax

#[mock::app(parse_binds)]
mod app {
    #[resources]
    struct Resources {
        a: u32,
        b: u32,
        #[init(0)]
        c: u32,
        #[init(0)]
        d: u32,
    }

    #[init(
        resources = [c],
    )]
    fn init(_: init::Context) -> init::LateResources {
        #[cfg(debug_assertions)]
        static mut X: u32 = 0;

        init::LateResources { a: 0, b: 0 }
    }

    #[idle(
        resources = [&a, d],
    )]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        loop {}
    }

    #[task(binds=[("cfg", I), ("ttt", T)],
        resources = [b, &c],
    )]
    fn foo(_: foo::Context) {
        static mut X: u32 = 0;

        *X += 1;
    }

    #[task(
        capacity = 2,
        priority = 2,
        resources = [d],
    )]
    fn bar(_: bar::Context, _: u32) {
        static mut X: u32 = 0;

        *X += 1;
    }
}
