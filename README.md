# RUSTIVITY

Rustivity es un Crate que implementa la gestión de estados, señales y efectos en Rust.

OJO: no está pensado para utilizar en entornos multi-threads de momento.

Está enfocado en que sea fácil de utilizar, ligero, y dependencias mínimas.

## Cómo se usa?

- Para empezar, añade el paquete en tu proyecto de Rust:
```sh
cargo add rustivity
```

- Crear un estado con un valor inicial:
```rs
let mut contador = use_state(0);
```

- Modificar el valor:
```rs
if let Ok(()) = contador.set(1){
    // TODO FUE BIEN!
}

// O TAMBIÉN
if let Ok(()) = contador.setter(|v| v+1){
    // TODO FUE BIEN!
}
```

- Manejar signals:
```rs
let mut contador = use_state(0);

// el id es a su vez un estado que cuando se elimine el elemento se pondrá en -1
let mut id = contador.signal(|state_value| {
    // aqui dentro no se puede modificar al estado
});

contador.set(1); // llama al signal con 1
contador.set(1); // no modifica el valor del estado

assert!(contador.rm_signal(&mut id)); // elimina el signal si el id es valido (sin dañar otros signals)

contador.flush_signals(); // los ids posiblemente dejen de funcionar algunos si eliminaste alguno antes
```

- Clonar estados:
```rs
// 1ra forma (clona el valor y los signals)
let mut new_state = old_state.clone();
// se debe clonar cada id para tener estados distintos del id para cada estado con esda signal
let mut new_signal_id = old_signal_id.clone();

// 2da forma (el new_from solo clona el valor, no los estados)
let mut new_state = StateObject<>::new_from(&old_state);
```

## Qué es un effect?

Un 'effect' es un concepto traído de otras librerías, qué básicamente es una función con dependencias hacia otros estados, qué se ejecutará al menos 1 vez y por cada cambio en los estados. La primera vez que se llama al effect, se usa el trait default para pasarle un valor por defecto.

Ej.
```rs
let mut state1 = use_state(0);
let mut state2 = use_state(1);

effect(
    |state| {
        // aquí dentro no se pueden modificar los estados!
    },
    vec![&mut state1, &mut state2],
);

state1.set(1).unwrap();
state2.set(0).unwrap();
```