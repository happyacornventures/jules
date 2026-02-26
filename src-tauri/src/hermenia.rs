use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct Machine {
    pub data: Mutex<HashMap<String, Value>>,
    pub reducers: HashMap<String, (Value, fn(Value, Value) -> Value)>,
    pub listeners: Mutex<Vec<Box<dyn Fn(&str, &Value, &Value) + Send + Sync>>>,
    pub interpreters: Mutex<Vec<Box<dyn Fn(&Value) -> Value + Send + Sync>>>,
}

impl Machine {
    pub fn new(
        data: HashMap<String, Value>,
        reducers: HashMap<String, (Value, fn(Value, Value) -> Value)>,
        listeners: Mutex<Vec<Box<dyn Fn(&str, &Value, &Value) + Send + Sync>>>,
    ) -> Self {
        Self {
            data: data.into(),
            reducers,
            listeners,
            interpreters: Mutex::new(Vec::new()),
        }
    }

    pub fn consume(&self, event: Value) -> HashMap<String, Value> {
        let mut data = self.data.lock().unwrap();
        let mut wrapped_event = event.clone();
        for interpreter in self.interpreters.lock().unwrap().iter() {
            wrapped_event = interpreter(&wrapped_event.clone());
        }

        for (key, value) in data.iter_mut() {
            if let Some((_initial_value, reducer)) = self.reducers.get(key) {
                let updated_value = reducer(value.clone(), wrapped_event.clone());
                if *value != updated_value {
                    *value = updated_value.clone();
                    for listener in self.listeners.lock().unwrap().iter() {
                        listener(key, &updated_value, &wrapped_event);
                    }
                }
            }
        }

        data.clone()
    }

    pub fn subscribe(&self, callback: Box<dyn Fn(&str, &Value, &Value) + Send + Sync>) {
        self.listeners.lock().unwrap().push(callback);
    }

    pub fn interpret(&self, callback: Box<dyn Fn(&Value) -> Value + Send + Sync>) {
        self.interpreters.lock().unwrap().push(callback);
    }
}
