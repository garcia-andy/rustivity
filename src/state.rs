use std::sync::{MutexGuard, PoisonError};

/// Enum for store signals on the vector and not modify id's
pub enum SignalType<T: Clone> {
    ValidSignal(fn(state: &T)),
    InvalidSignal,
}

/// Clone Implementation for SignalTypes (Copy Func)
impl <T: Clone>Clone for SignalType<T> {
    fn clone(&self) -> Self {
        if let SignalType::ValidSignal(func) = self{
            SignalType::ValidSignal(func.clone())
        }else{ SignalType::InvalidSignal }
    }
}

/// The State Object, with the Mutex for async access and the signals list
pub struct StateObject<T: Clone+ PartialEq> {
    value: std::sync::Mutex<T>,
    signals: Vec<SignalType<T>>,
}

/// State Object implementation for the T with Clone trait
impl<T: Clone + PartialEq> StateObject<T> {
    
    /// Create a new state with the value provided
    pub fn new(initial_value: T) -> Self {
        Self {
            value: std::sync::Mutex::new(initial_value),
            signals: Vec::new(),
        }
    }
    
    /// Create a new state from the value of another state
    pub fn new_from(other: &Self) -> Self{
        Self::new(other.get())
    }

    /// Set the new value of the internal state (& invoke the signals with a cloned state value)
    pub fn set(&mut self, new_value: T) -> Result<(), PoisonError<MutexGuard<'_, T>>> {
        let mut wrapped = self.value.lock()?;
        if (*wrapped) != new_value{
            *wrapped = new_value;
            
            for clbks in self.signals.iter(){
                if let SignalType::ValidSignal(func) = clbks{
                    (*func)(&*wrapped)
                }
            }
        }
        
        Ok(())
    }
    
    pub fn setter(&mut self, fn_setter: fn(state: T) -> T) -> Result<(), PoisonError<MutexGuard<'_, T>>> {
        let value = self.get();
        self.set(fn_setter(value))
    }

    /// Get a clone of the state value
    pub fn get(&self) -> T {
        self.value.lock().unwrap().clone()
    }

    /// Save a external created Signal pointer impl
    pub fn signal(&mut self, sign: fn(state: &T)) -> StateObject<isize> {
        self.signals.push(SignalType::ValidSignal(sign));
        StateObject::new( (self.signals.len() - 1) as isize )
    }

    /// Remove an existent signal (just set to InvalidSignal)
    pub fn rm_signal(&mut self, id: &mut StateObject<isize>) -> bool {
        if self.signals.len() as isize > id.get() && id.get() > -1 {
            self.signals[id.get() as usize] = SignalType::InvalidSignal;
            id.set(-1).expect("Error setting new id!");
            true
        } else {
            false
        }
    }
    
    /// Delete every SignalType::InvalidSignal from the signals vector
    /// Maybe ids broken but in some cases more efficent callbacks
    pub fn flush_signals(&mut self) -> u32{
        let mut count = 0u32;
        let newvec: Vec<SignalType<T>> = self.signals.iter().filter(|s|{
            if let SignalType::InvalidSignal = s{
                count += 1;
                false
            }else{true}
        }).cloned().collect();
        self.signals = newvec;
        count
    }
}

/// Implementation of Clone for the StateObject
impl <T: Clone + PartialEq> Clone for StateObject<T>{
    fn clone(&self) -> Self {
        let v = self.get();
        let mut this = StateObject::new(v);
        
        for sing in self.signals.iter(){
            if let SignalType::ValidSignal(func) = sing{
                this.signal(*func);
            }else{
                // for valids ids
                this.signals.push(SignalType::InvalidSignal);
            }
        }
        
        this
    }
}

/// Function for automatic signals & first-call functions creation
pub fn effect<T: Default + Clone + PartialEq>(callback: fn(&T), mut depends: Vec<&mut StateObject<T>>) {
    (callback)(&T::default());
    depends.iter_mut().for_each(|s| {
        s.signal(callback);
    });
}

/// Utility function for create new state objects
pub fn use_state<T: Clone + PartialEq>(initial_value: T) -> StateObject<T> {
    StateObject::new(initial_value)
}

#[cfg(test)]
mod test {
    use super::{effect, use_state, StateObject};

    #[test]
    fn effect_test() {
        let mut counter = 0;
        let mut state1 = use_state(counter);
        counter += 1;
        let mut state2 = use_state(counter);

        effect(
            |state| assert!((*state == 0) || (*state == 1)),
            vec![&mut state1, &mut state2],
        );

        state1.set(1).unwrap();
        state2.set(0).unwrap();
    }

    #[test]
    fn clone_test() {
        let counter = 0;
        let mut state = use_state(counter);
        assert_eq!(state.get(), counter);
        
        let mut id = state.signal(|state| {
            assert_ne!(*state, -1);
        });
        
        let mut id2 = id.clone();
        
        state.set(2).unwrap();
        
        let mut state2 = state.clone();
        let state3 = StateObject::new_from(&state);
        
        // Checking independence value
        assert_eq!(state2.get(),state.get());
        assert_eq!(state3.get(),state.get());
        state2.set(3).unwrap();
        assert_ne!(state2.get(),state.get());
        assert_eq!(state3.get(),state.get());
        
        
        // Checking Independence signals (but copieds)
        assert!(state.rm_signal(&mut id));
        state.set(-1).unwrap();
        
        assert!(state2.rm_signal(&mut id2));
        state2.set(-1).unwrap();
    }
    
    #[test]
    fn state_test() {
        let mut counter = 0;
        let mut state = use_state(counter);
        assert_eq!(state.get(), counter);
        
        counter = state.get();
        state.set(counter + 2).unwrap();
        assert_ne!(counter, state.get());
        assert_eq!(counter + 2, state.get());
        
        counter = state.get();
        state.setter(|v| v+1).expect("No se pudo acceder al mutex lock!");
        assert_ne!(counter, state.get());
    }

    #[test]
    fn signal_test() {
        let mut counter = 0;
        let mut state = use_state(counter);
        let mut id = state.signal(|state| {
            assert_ne!(*state, -1);
        });

        assert_eq!(state.get(), counter);
        counter = state.get();
        state.set(counter + 2).unwrap();
        assert_ne!(counter, state.get());
        assert_eq!(counter + 2, state.get());

        let mut id2 = state.signal(|state| {
            assert_eq!(*state, -1);
        });

        state.rm_signal(&mut id);
        state.set(-1).unwrap();
        assert_eq!(state.get(), -1);
        // here fail! (id2 broked if the indeces change)
        // state.flush_signals();
        assert!(state.rm_signal(&mut id2));
        state.flush_signals(); // this works properly
        
        state.set(2).unwrap();
        assert_eq!(state.get(), 2);
    }
}
