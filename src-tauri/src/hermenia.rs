use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct Machine {
    pub data: Mutex<HashMap<String, Value>>,
    pub reducers: HashMap<String, (Value, fn(Value, Value) -> Value)>,
    pub listeners: Mutex<Vec<Box<dyn Fn(&str, &Value, &Value) + Send + Sync>>>,
}

fn hydrate_event(event: String, payload: &str) -> Value {
    let id = Uuid::new_v4().to_string();
    let payload_value: Value = serde_json::from_str(payload).unwrap();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    json!({
        "id": id,
        "createTime": timestamp,
        "type": event,
        "payload": payload_value
    })
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
        }
    }

    pub fn consume(&self, event: String, payload: Option<String>) -> String {
        let mut data = self.data.lock().unwrap();
        let payload_str = payload.as_deref().unwrap_or("{}");
        let hydrated_event = hydrate_event(event.to_string(), payload_str);

        for (key, value) in data.iter_mut() {
            if let Some((_initial_value, reducer)) = self.reducers.get(key) {
                let updated_value = reducer(value.clone(), hydrated_event.clone());
                if *value != updated_value {
                    *value = updated_value.clone();
                    for listener in self.listeners.lock().unwrap().iter() {
                        listener(key, &updated_value, &hydrated_event);
                    }
                }
            }
        }

        serde_json::to_string(&*data).unwrap()
    }

    pub fn subscribe(&self, callback: Box<dyn Fn(&str, &Value, &Value) + Send + Sync>) {
        self.listeners.lock().unwrap().push(callback);
    }
}
