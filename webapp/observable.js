export function observable(initialValue) {
    let _value = initialValue;
    const listeners = new Set();
  
    return {
      get value() {
        return _value;
      },
      set value(newVal) {
        if (_value !== newVal) {
          _value = newVal;
          listeners.forEach(fn => fn(newVal));
        }
      },
      subscribe(fn) {
        listeners.add(fn);
        fn(_value); // immediately notify with current value
        return () => listeners.delete(fn); // unsubscribe
      }
    };
  }
  