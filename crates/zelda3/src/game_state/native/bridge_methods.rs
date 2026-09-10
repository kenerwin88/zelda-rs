//! Mechanical forwarding for native bridges with an eager RAM sync.
//!
//! Keep the state writers and `sync` methods explicit: they define which bytes
//! each bridge owns. Use this only when arguments pass through unchanged and
//! the state call is immediately followed by exactly one sync.

macro_rules! forward_synced {
    ($field:ident; $(fn $name:ident($($arg:ident: $arg_type:ty),* $(,)?) $(-> $ret:ty)?;)+) => {
        $(forward_synced!(@method $field; fn $name($($arg: $arg_type),*) $(-> $ret)?);)+
    };
    (@method $field:ident; fn $name:ident($($arg:ident: $arg_type:ty),*)) => {
        pub(crate) fn $name(&mut self, $($arg: $arg_type),*) {
            self.$field.$name($($arg),*);
            self.sync();
        }
    };
    (@method $field:ident; fn $name:ident($($arg:ident: $arg_type:ty),*) -> $ret:ty) => {
        pub(crate) fn $name(&mut self, $($arg: $arg_type),*) -> $ret {
            let result = self.$field.$name($($arg),*);
            self.sync();
            result
        }
    };
}

#[cfg(test)]
mod tests {
    struct State {
        value: u8,
    }

    impl State {
        fn set_value(&mut self, value: u8) -> bool {
            self.value = value;
            true
        }

        fn advance(&mut self, delta: u8) -> u8 {
            self.value = self.value.wrapping_add(delta);
            self.value
        }
    }

    struct Bridge {
        state: State,
        synced_values: Vec<u8>,
    }

    impl Bridge {
        forward_synced! {
            state;
            fn set_value(value: u8);
            fn advance(delta: u8) -> u8;
        }

        fn sync(&mut self) {
            self.synced_values.push(self.state.value);
            // Make the return-before-sync contract observable.
            self.state.value = 0;
        }
    }

    #[test]
    fn void_forwarding_discards_the_state_result_and_syncs_once_after_the_write() {
        let mut bridge = Bridge {
            state: State { value: 0 },
            synced_values: Vec::new(),
        };
        bridge.set_value(17);
        assert_eq!(bridge.synced_values, [17]);
        assert_eq!(bridge.state.value, 0);
    }

    #[test]
    fn returning_forwarding_returns_the_call_result_before_sync() {
        let mut bridge = Bridge {
            state: State { value: 250 },
            synced_values: Vec::new(),
        };
        assert_eq!(bridge.advance(10), 4);
        assert_eq!(bridge.synced_values, [4]);
        assert_eq!(bridge.state.value, 0);
    }
}

/// A bridge over one native state: it adopts the state from live WRAM when it is
/// constructed, and every synced setter projects the state through a compare-on-write
/// target so only the bytes the mutation changed are stored. The bridge's own setters
/// live in a separate `impl` block next to the invocation.
macro_rules! adopting_bridge {
    ($bridge:ident, $field:ident: $state:ty) => {
        pub(crate) struct $bridge<'a> {
            $field: &'a mut $state,
            ram: &'a mut [u8],
        }

        impl<'a> $bridge<'a> {
            pub(crate) fn new($field: &'a mut $state, ram: &'a mut [u8]) -> Self {
                *$field = <$state>::load_from_ram(&*ram);
                Self { $field, ram }
            }

            fn sync(&mut self) {
                self.$field.write_to_ram(
                    &mut crate::game_state::native::ram_target::DiffTarget::new(self.ram),
                );
                self.debug_assert_matches_ram();
            }

            fn debug_assert_matches_ram(&self) {
                debug_assert_eq!(*self.$field, <$state>::load_from_ram(self.ram));
            }
        }
    };
}
